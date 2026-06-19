use crate::ast::*;
use crate::error::SpandaError;
use crate::foundations::*;
use crate::lexer::{unit_from_lexeme, Token, TokenType, TokenValue, UnitLexeme};

pub fn parse(tokens: Vec<Token>) -> Result<Program, SpandaError> {
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    fn advance(&mut self) -> Token {
        if self.peek().token_type != TokenType::Eof {
            self.pos += 1;
        }
        self.tokens[self.pos - 1].clone()
    }

    fn check(&self, ty: TokenType) -> bool {
        self.peek().token_type == ty
    }

    fn match_types(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(*t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expect(&mut self, ty: TokenType, message: &str) -> Result<Token, SpandaError> {
        if self.check(ty) {
            Ok(self.advance())
        } else {
            let t = self.peek();
            Err(SpandaError::Parse {
                message: message.to_string(),
                line: t.line,
                column: t.column,
            })
        }
    }

    fn span_from(&self, start: &Token, end: &Token) -> Span {
        Span {
            start: loc(start),
            end: loc(end),
        }
    }

    fn parse_label(&mut self, message: &str) -> Result<String, SpandaError> {
        let t = self.peek();
        let ok = matches!(
            t.token_type,
            TokenType::Ident
                | TokenType::Plan
                | TokenType::Twin
                | TokenType::Skill
                | TokenType::Match
                | TokenType::State
                | TokenType::Event
                | TokenType::Task
                | TokenType::Action
                | TokenType::Goal
                | TokenType::Memory
                | TokenType::On
                | TokenType::Replay
                | TokenType::Mirror
                | TokenType::Enter
                | TokenType::Emit
        );
        if ok {
            Ok(self.advance().lexeme)
        } else {
            Err(SpandaError::Parse {
                message: message.to_string(),
                line: t.line,
                column: t.column,
            })
        }
    }

    fn parse_program(&mut self) -> Result<Program, SpandaError> {
        let start = self.peek().clone();
        let mut module_name = None;
        if self.check(TokenType::Module) {
            self.advance();
            module_name = Some(self.parse_label("Expected module name after 'module'")?);
            self.expect(TokenType::Semicolon, "Expected ';' after module declaration")?;
        }
        let mut imports = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut robots = Vec::new();
        while self.check(TokenType::Import) {
            imports.push(self.parse_import()?);
        }
        while !self.check(TokenType::Eof) {
            if self.check(TokenType::Struct) {
                structs.push(self.parse_struct()?);
            } else if self.check(TokenType::Enum) {
                enums.push(self.parse_enum()?);
            } else if self.check(TokenType::Trait) {
                traits.push(self.parse_trait()?);
            } else if self.check(TokenType::Robot) {
                robots.push(self.parse_robot()?);
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected struct, enum, trait, or robot declaration".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        Ok(Program::Program {
            module_name,
            imports,
            structs,
            enums,
            traits,
            robots,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_import(&mut self) -> Result<ImportDecl, SpandaError> {
        let start = self.advance();
        let vendor = self.parse_label("Expected library vendor name")?;
        self.expect(TokenType::Dot, "Expected '.' in import path")?;
        let module = self.parse_label("Expected library module name")?;
        self.expect(TokenType::Semicolon, "Expected ';' after import")?;
        Ok(ImportDecl::ImportDecl {
            path: format!("{}.{}", vendor, module),
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_struct(&mut self) -> Result<StructDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected struct name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after struct name")?;
        let mut fields = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let field_start = self.peek().clone();
            let field_name = self.expect(TokenType::Ident, "Expected field name")?;
            self.expect(TokenType::Colon, "Expected ':' after field name")?;
            let type_name = self.expect(TokenType::Ident, "Expected field type")?;
            self.expect(TokenType::Semicolon, "Expected ';' after field")?;
            fields.push(FieldDecl {
                name: field_name.lexeme,
                type_name: type_name.lexeme,
                span: self.span_from(&field_start, self.previous()),
            });
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close struct")?;
        Ok(StructDecl::StructDecl {
            name: name.lexeme,
            fields,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_enum(&mut self) -> Result<EnumDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected enum name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after enum name")?;
        let mut variants = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            variants.push(self.expect(TokenType::Ident, "Expected enum variant")?.lexeme);
            if self.match_types(&[TokenType::Comma]) {
                continue;
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close enum")?;
        Ok(EnumDecl::EnumDecl {
            name: name.lexeme,
            variants,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_trait(&mut self) -> Result<TraitDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected trait name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after trait name")?;
        let mut methods = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            methods.push(self.parse_trait_method()?);
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close trait")?;
        Ok(TraitDecl::TraitDecl {
            name: name.lexeme,
            methods,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_trait_method(&mut self) -> Result<TraitMethodDecl, SpandaError> {
        let start = self.advance(); // fn
        let name = self.parse_label("Expected method name after fn")?;
        self.expect(TokenType::Lparen, "Expected '(' after method name")?;
        let mut params = Vec::new();
        if !self.check(TokenType::Rparen) {
            loop {
                let param_start = self.peek().clone();
                let param_name = self.parse_label("Expected parameter name")?;
                self.expect(TokenType::Colon, "Expected ':' after parameter name")?;
                let type_name = self.expect(TokenType::Ident, "Expected parameter type")?;
                params.push(TraitParamDecl {
                    name: param_name,
                    type_name: type_name.lexeme,
                    span: self.span_from(&param_start, self.previous()),
                });
                if !self.match_types(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.expect(TokenType::Rparen, "Expected ')' after parameters")?;
        self.expect(TokenType::Arrow, "Expected '->' after trait method parameters")?;
        let return_type = self.expect(TokenType::Ident, "Expected return type")?;
        self.expect(TokenType::Semicolon, "Expected ';' after trait method")?;
        Ok(TraitMethodDecl {
            name,
            params,
            return_type: return_type.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_robot(&mut self) -> Result<RobotDecl, SpandaError> {
        let start = self.expect(TokenType::Robot, "Expected 'robot'")?;
        let name_tok = self.expect(TokenType::Ident, "Expected robot name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after robot name")?;
        let mut soc = None;
        let mut hal = None;
        let mut nodes = Vec::new();
        let mut topics = Vec::new();
        let mut services = Vec::new();
        let mut actions = Vec::new();
        let mut sensors = Vec::new();
        let mut actuators = Vec::new();
        let mut safety = None;
        let mut ai_models = Vec::new();
        let mut agents = Vec::new();
        let mut behaviors = Vec::new();
        let mut tasks = Vec::new();
        let mut state_machines = Vec::new();
        let mut events = Vec::new();
        let mut event_handlers = Vec::new();
        let mut twin = None;
        let mut trait_impls = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.check(TokenType::Soc) {
                soc = Some(self.parse_soc()?);
            } else if self.check(TokenType::Hal) {
                hal = Some(self.parse_hal()?);
            } else if self.check(TokenType::Node) {
                nodes.push(self.parse_node()?);
            } else if self.check(TokenType::Topic) {
                topics.push(self.parse_topic()?);
            } else if self.check(TokenType::Service) {
                services.push(self.parse_service()?);
            } else if self.check(TokenType::Action) {
                actions.push(self.parse_action()?);
            } else if self.check(TokenType::Sensor) {
                sensors.push(self.parse_sensor()?);
            } else if self.check(TokenType::Actuator) {
                actuators.push(self.parse_actuator()?);
            } else if self.check(TokenType::Safety) {
                safety = Some(self.parse_safety()?);
            } else if self.check(TokenType::AiModel) {
                ai_models.push(self.parse_ai_model()?);
            } else if self.check(TokenType::Agent) {
                agents.push(self.parse_agent()?);
            } else if self.check(TokenType::Behavior) {
                behaviors.push(self.parse_behavior()?);
            } else if self.check(TokenType::Task) {
                tasks.push(self.parse_task()?);
            } else if self.check(TokenType::StateMachine) {
                state_machines.push(self.parse_state_machine()?);
            } else if self.check(TokenType::Event) {
                events.push(self.parse_event()?);
            } else if self.check(TokenType::On) {
                event_handlers.push(self.parse_event_handler()?);
            } else if self.check(TokenType::Twin) {
                twin = Some(self.parse_twin()?);
            } else if self.check(TokenType::Impl) {
                trait_impls.push(self.parse_trait_impl()?);
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected robot member declaration".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close robot block")?;
        Ok(RobotDecl::RobotDecl {
            name: name_tok.lexeme,
            soc,
            hal,
            nodes,
            topics,
            services,
            actions,
            sensors,
            actuators,
            safety,
            ai_models,
            agents,
            behaviors,
            tasks,
            state_machines,
            events,
            event_handlers,
            twin,
            trait_impls,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_trait_impl(&mut self) -> Result<crate::foundations::TraitImplDecl, SpandaError> {
        use crate::foundations::TraitImplDecl;
        let start = self.expect(TokenType::Impl, "Expected 'impl'")?;
        let trait_name = self.parse_label("Expected trait name after 'impl'")?;
        self.expect(TokenType::For, "Expected 'for' after trait name")?;
        let agent_name = self.parse_label("Expected agent name after 'for'")?;
        self.expect(TokenType::Lbrace, "Expected '{' after trait impl header")?;
        let mut methods = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            methods.push(self.parse_trait_impl_method()?);
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close trait impl")?;
        Ok(TraitImplDecl::TraitImplDecl {
            trait_name,
            agent_name,
            methods,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_trait_impl_method(&mut self) -> Result<crate::foundations::TraitImplMethodDecl, SpandaError> {
        use crate::foundations::TraitImplMethodDecl;
        let start = self.expect(TokenType::Fn, "Expected 'fn' in trait impl method")?;
        let name = self.parse_label("Expected method name")?;
        self.expect(TokenType::Lparen, "Expected '(' after method name")?;
        let mut params = Vec::new();
        if !self.check(TokenType::Rparen) {
            loop {
                let pstart = self.peek().clone();
                let pname = self.parse_label("Expected parameter name")?;
                self.expect(TokenType::Colon, "Expected ':' after parameter name")?;
                let ptype = self.expect(TokenType::Ident, "Expected parameter type")?.lexeme;
                params.push(crate::foundations::TraitParamDecl {
                    name: pname,
                    type_name: ptype,
                    span: self.span_from(&pstart, self.previous()),
                });
                if !self.match_types(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.expect(TokenType::Rparen, "Expected ')' after parameters")?;
        self.expect(TokenType::Arrow, "Expected '->' after trait impl parameters")?;
        let return_type = self.expect(TokenType::Ident, "Expected return type")?.lexeme;
        self.expect(TokenType::Lbrace, "Expected '{' after trait impl method signature")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close trait impl method")?;
        Ok(TraitImplMethodDecl {
            name,
            params,
            return_type,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_soc(&mut self) -> Result<SocDecl, SpandaError> {
        let start = self.advance();
        let profile = self.expect(TokenType::Ident, "Expected SoC profile name")?;
        self.expect(TokenType::Semicolon, "Expected ';' after soc declaration")?;
        Ok(SocDecl::SocDecl {
            profile: profile.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_hal(&mut self) -> Result<HalBlock, SpandaError> {
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after hal")?;
        let mut members = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            members.push(self.parse_hal_member()?);
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close hal block")?;
        Ok(HalBlock::HalBlock {
            members,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_hal_member(&mut self) -> Result<HalMemberDecl, SpandaError> {
        let start = self.peek().clone();
        if self.match_types(&[TokenType::I2c]) {
            let name = self.expect(TokenType::Ident, "Expected I2C bus name")?;
            self.expect(TokenType::At, "Expected 'at' after I2C bus name")?;
            let addr = self.expect(TokenType::Number, "Expected I2C address")?;
            self.expect(TokenType::Semicolon, "Expected ';' after I2C declaration")?;
            return Ok(HalMemberDecl::HalI2cDecl {
                name: name.lexeme,
                address: num(&addr),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Spi]) {
            let name = self.expect(TokenType::Ident, "Expected SPI bus name")?;
            self.expect(TokenType::At, "Expected 'at' after SPI bus name")?;
            let bus = self.expect(TokenType::Number, "Expected SPI bus number")?;
            let mut cs_pin = None;
            if self.match_types(&[TokenType::Pin]) {
                cs_pin = Some(num(&self.expect(TokenType::Number, "Expected CS pin number")?));
            }
            self.expect(TokenType::Semicolon, "Expected ';' after SPI declaration")?;
            return Ok(HalMemberDecl::HalSpiDecl {
                name: name.lexeme,
                bus: num(&bus),
                cs_pin,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Gpio]) {
            let name = self.expect(TokenType::Ident, "Expected GPIO name")?;
            let direction = if self.match_types(&[TokenType::Out]) {
                GpioDirection::Out
            } else if self.match_types(&[TokenType::In]) {
                GpioDirection::In
            } else {
                GpioDirection::Out
            };
            self.expect(TokenType::Pin, "Expected 'pin' keyword")?;
            let pin = self.expect(TokenType::Number, "Expected GPIO pin number")?;
            self.expect(TokenType::Semicolon, "Expected ';' after GPIO declaration")?;
            return Ok(HalMemberDecl::HalGpioDecl {
                name: name.lexeme,
                direction,
                pin: num(&pin),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Pwm]) {
            let name = self.expect(TokenType::Ident, "Expected PWM name")?;
            self.expect(TokenType::On, "Expected 'on' after PWM name")?;
            self.expect(TokenType::Pin, "Expected 'pin' after on")?;
            let pin = self.expect(TokenType::Number, "Expected PWM pin")?;
            self.expect(TokenType::Frequency, "Expected 'frequency' after PWM pin")?;
            let freq = self.parse_frequency_hz()?;
            self.expect(TokenType::Semicolon, "Expected ';' after PWM declaration")?;
            return Ok(HalMemberDecl::HalPwmDecl {
                name: name.lexeme,
                pin: num(&pin),
                frequency_hz: freq,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Uart]) {
            let name = self.expect(TokenType::Ident, "Expected UART name")?;
            self.expect(TokenType::On, "Expected 'on' after UART name")?;
            let device = self.expect(TokenType::String, "Expected UART device path")?;
            self.expect(TokenType::Baud, "Expected 'baud' after UART device")?;
            let baud = self.expect(TokenType::Number, "Expected baud rate")?;
            self.expect(TokenType::Semicolon, "Expected ';' after UART declaration")?;
            return Ok(HalMemberDecl::HalUartDecl {
                name: name.lexeme,
                device: str_val(&device),
                baud: num(&baud),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Adc]) {
            let name = self.expect(TokenType::Ident, "Expected ADC name")?;
            self.expect(TokenType::On, "Expected 'on' after ADC name")?;
            self.expect(TokenType::Ident, "Expected 'channel' keyword")?;
            let ch = self.expect(TokenType::Number, "Expected ADC channel number")?;
            self.expect(TokenType::Semicolon, "Expected ';' after ADC declaration")?;
            return Ok(HalMemberDecl::HalAdcDecl {
                name: name.lexeme,
                channel: num(&ch),
                span: self.span_from(&start, self.previous()),
            });
        }
        let t = self.peek();
        Err(SpandaError::Parse {
            message: "Expected HAL member (i2c, spi, gpio, pwm, uart, adc)".into(),
            line: t.line,
            column: t.column,
        })
    }

    fn parse_frequency_hz(&mut self) -> Result<f64, SpandaError> {
        let tok = self.peek().clone();
        if tok.token_type == TokenType::UnitLiteral && tok.unit == Some(UnitLexeme::Hz) {
            self.advance();
            return Ok(num(&tok));
        }
        if tok.token_type == TokenType::Number {
            self.advance();
            if self.check(TokenType::Ident) && self.peek().lexeme == "Hz" {
                self.advance();
            }
            return Ok(num(&tok));
        }
        Err(SpandaError::Parse {
            message: "Expected frequency like 50 Hz".into(),
            line: tok.line,
            column: tok.column,
        })
    }

    fn parse_node(&mut self) -> Result<NodeDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected node name")?;
        let namespace = if self.match_types(&[TokenType::On]) {
            Some(str_val(&self.expect(TokenType::String, "Expected namespace string after 'on'")?))
        } else {
            None
        };
        self.expect(TokenType::Semicolon, "Expected ';' after node declaration")?;
        Ok(NodeDecl::NodeDecl {
            name: name.lexeme,
            namespace,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_topic(&mut self) -> Result<TopicDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected topic name")?;
        self.expect(TokenType::Colon, "Expected ':' after topic name")?;
        let message_type = self.expect(TokenType::Ident, "Expected message type")?;
        self.expect(TokenType::Publish, "Expected 'publish' after message type")?;
        self.expect(TokenType::On, "Expected 'on' after publish")?;
        let topic = self.expect(TokenType::String, "Expected topic string")?;
        self.expect(TokenType::Semicolon, "Expected ';' after topic declaration")?;
        Ok(TopicDecl::TopicDecl {
            name: name.lexeme,
            message_type: message_type.lexeme,
            topic: str_val(&topic),
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_service(&mut self) -> Result<ServiceDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected service name")?;
        self.expect(TokenType::Colon, "Expected ':' after service name")?;
        let service_type = self.expect(TokenType::Ident, "Expected service type")?;
        self.expect(TokenType::Semicolon, "Expected ';' after service declaration")?;
        Ok(ServiceDecl::ServiceDecl {
            name: name.lexeme,
            service_type: service_type.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_action(&mut self) -> Result<ActionDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected action name")?;
        self.expect(TokenType::Colon, "Expected ':' after action name")?;
        let action_type = self.expect(TokenType::Ident, "Expected action type")?;
        self.expect(TokenType::Semicolon, "Expected ';' after action declaration")?;
        Ok(ActionDecl::ActionDecl {
            name: name.lexeme,
            action_type: action_type.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_sensor(&mut self) -> Result<SensorDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected sensor name")?;
        self.expect(TokenType::Colon, "Expected ':' after sensor name")?;
        let sensor_type = self.expect(TokenType::Ident, "Expected sensor type")?;
        let library = if self.match_types(&[TokenType::From]) {
            let vendor = self.expect(TokenType::Ident, "Expected library vendor in from clause")?;
            self.expect(TokenType::Dot, "Expected '.' in library path")?;
            let module = self.expect(TokenType::Ident, "Expected library module in from clause")?;
            Some(format!("{}.{}", vendor.lexeme, module.lexeme))
        } else {
            None
        };
        let binding = if self.match_types(&[TokenType::On]) {
            if self.check(TokenType::String) {
                Some(SensorBinding::Topic {
                    path: str_val(&self.advance()),
                })
            } else {
                Some(SensorBinding::Hal {
                    bus_name: self
                        .expect(TokenType::Ident, "Expected HAL bus name or topic string after 'on'")?
                        .lexeme,
                })
            }
        } else {
            None
        };
        self.expect(TokenType::Semicolon, "Expected ';' after sensor declaration")?;
        Ok(SensorDecl::SensorDecl {
            name: name.lexeme,
            sensor_type: sensor_type.lexeme,
            library,
            binding,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_actuator(&mut self) -> Result<ActuatorDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected actuator name")?;
        self.expect(TokenType::Colon, "Expected ':' after actuator name")?;
        let actuator_type = self.expect(TokenType::Ident, "Expected actuator type")?;
        self.expect(TokenType::Semicolon, "Expected ';' after actuator declaration")?;
        Ok(ActuatorDecl::ActuatorDecl {
            name: name.lexeme,
            actuator_type: actuator_type.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_safety(&mut self) -> Result<SafetyBlock, SpandaError> {
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after safety")?;
        let mut rules = Vec::new();
        let mut zones = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.check(TokenType::StopIf) {
                rules.push(self.parse_stop_if_rule()?);
            } else if self.check(TokenType::Zone) {
                zones.push(self.parse_safety_zone()?);
            } else if self.check(TokenType::Ident) {
                rules.push(self.parse_max_speed_rule()?);
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected safety rule or zone".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close safety block")?;
        Ok(SafetyBlock::SafetyBlock {
            rules,
            zones,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_ai_model(&mut self) -> Result<AiModelDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected ai model name")?;
        self.expect(TokenType::Colon, "Expected ':' after ai model name")?;
        let model_type = self.expect(TokenType::Ident, "Expected ai model type")?;
        self.expect(TokenType::Lbrace, "Expected '{' after ai model type")?;
        let config = self.parse_ai_config_entries()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close ai model config")?;
        Ok(AiModelDecl::AiModelDecl {
            name: name.lexeme,
            model_type: model_type.lexeme,
            config,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_ai_config_entries(&mut self) -> Result<Vec<AiConfigEntry>, SpandaError> {
        let mut entries = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let entry_start = self.peek().clone();
            let key = self.parse_config_key_token()?;
            self.expect(TokenType::Colon, "Expected ':' in ai model config")?;
            let value = self.parse_config_value()?;
            self.expect(TokenType::Semicolon, "Expected ';' after ai model config entry")?;
            entries.push(AiConfigEntry {
                key,
                value,
                span: self.span_from(&entry_start, self.previous()),
            });
        }
        Ok(entries)
    }

    fn parse_config_key_token(&mut self) -> Result<String, SpandaError> {
        if self.check(TokenType::Ident) || self.check(TokenType::Provider) {
            Ok(self.advance().lexeme)
        } else {
            let t = self.peek();
            Err(SpandaError::Parse {
                message: "Expected config key".into(),
                line: t.line,
                column: t.column,
            })
        }
    }

    fn parse_config_value(&mut self) -> Result<ConfigValue, SpandaError> {
        if self.match_types(&[TokenType::String]) {
            Ok(ConfigValue::String(str_val(self.previous())))
        } else if self.match_types(&[TokenType::True]) {
            Ok(ConfigValue::Bool(true))
        } else if self.match_types(&[TokenType::False]) {
            Ok(ConfigValue::Bool(false))
        } else if self.match_types(&[TokenType::Number, TokenType::UnitLiteral]) {
            Ok(ConfigValue::Number(num(self.previous())))
        } else {
            let t = self.peek();
            Err(SpandaError::Parse {
                message: "Expected config value".into(),
                line: t.line,
                column: t.column,
            })
        }
    }

    fn parse_agent(&mut self) -> Result<AgentDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected agent name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after agent name")?;
        let mut uses_ai = Vec::new();
        let mut memory_kind = None;
        let mut tools = Vec::new();
        let mut skills = Vec::new();
        let mut capabilities = Vec::new();
        let mut goal = String::new();
        let mut plan_body = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Uses]) {
                uses_ai.push(self.expect(TokenType::Ident, "Expected model name after uses")?.lexeme);
                self.expect(TokenType::Semicolon, "Expected ';' after uses")?;
            } else if self.match_types(&[TokenType::Memory]) {
                let kind = self.expect(TokenType::Ident, "Expected memory kind")?;
                memory_kind = Some(match kind.lexeme.as_str() {
                    "short_term" => MemoryKind::ShortTerm,
                    "long_term" => MemoryKind::LongTerm,
                    _ => {
                        return Err(SpandaError::Parse {
                            message: "Memory kind must be short_term or long_term".into(),
                            line: kind.line,
                            column: kind.column,
                        });
                    }
                });
                self.expect(TokenType::Semicolon, "Expected ';' after memory")?;
            } else if self.match_types(&[TokenType::Tools]) {
                self.expect(TokenType::Lbracket, "Expected '[' after tools")?;
                if !self.check(TokenType::Rbracket) {
                    loop {
                        tools.push(self.expect(TokenType::Ident, "Expected tool name")?.lexeme);
                        if !self.match_types(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.expect(TokenType::Rbracket, "Expected ']' after tools list")?;
                self.expect(TokenType::Semicolon, "Expected ';' after tools")?;
            } else if self.match_types(&[TokenType::Skill]) {
                skills.push(self.expect(TokenType::Ident, "Expected skill name")?.lexeme);
                self.expect(TokenType::Semicolon, "Expected ';' after skill")?;
            } else if self.match_types(&[TokenType::Can]) {
                self.expect(TokenType::Lbracket, "Expected '[' after can")?;
                if !self.check(TokenType::Rbracket) {
                    loop {
                        capabilities.push(self.parse_capability()?);
                        if !self.match_types(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.expect(TokenType::Rbracket, "Expected ']' after capability list")?;
                self.expect(TokenType::Semicolon, "Expected ';' after can")?;
            } else if self.match_types(&[TokenType::Goal]) {
                goal = str_val(&self.expect(TokenType::String, "Expected goal string")?);
                self.expect(TokenType::Semicolon, "Expected ';' after goal")?;
            } else if self.match_types(&[TokenType::Plan]) {
                self.expect(TokenType::Lbrace, "Expected '{' after plan")?;
                plan_body = self.parse_block()?;
                self.expect(TokenType::Rbrace, "Expected '}' to close plan")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected agent member".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close agent block")?;
        Ok(AgentDecl::AgentDecl {
            name: name.lexeme,
            uses_ai,
            memory_kind,
            tools,
            skills,
            capabilities,
            goal,
            plan_body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_capability(&mut self) -> Result<CapabilityDecl, SpandaError> {
        let start = self.peek().clone();
        let action = self.expect(TokenType::Ident, "Expected capability action")?;
        let target = if self.match_types(&[TokenType::Lparen]) {
            let t = self.expect(TokenType::Ident, "Expected capability target")?;
            self.expect(TokenType::Rparen, "Expected ')' after capability target")?;
            Some(t.lexeme)
        } else {
            None
        };
        Ok(CapabilityDecl {
            action: action.lexeme,
            target,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_safety_zone(&mut self) -> Result<SafetyZoneDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected zone name")?;
        let shape = if self.match_types(&[TokenType::Circle]) {
            ZoneShape::Circle
        } else if self.match_types(&[TokenType::Rect]) {
            ZoneShape::Rect
        } else {
            let t = self.peek();
            return Err(SpandaError::Parse {
                message: "Expected 'circle' or 'rect' after zone name".into(),
                line: t.line,
                column: t.column,
            });
        };
        self.expect(TokenType::At, "Expected 'at' in zone declaration")?;
        self.expect(TokenType::Lparen, "Expected '(' after 'at'")?;
        let x = self.parse_expr()?;
        self.expect(TokenType::Comma, "Expected ',' between coordinates")?;
        let y = self.parse_expr()?;
        self.expect(TokenType::Rparen, "Expected ')' after coordinates")?;
        let (radius, width, height) = if shape == ZoneShape::Circle {
            self.expect(TokenType::Radius, "Expected 'radius' for circle zone")?;
            (Some(self.parse_expr()?), None, None)
        } else {
            self.expect(TokenType::Size, "Expected 'size' for rect zone")?;
            self.expect(TokenType::Lparen, "Expected '(' after 'size'")?;
            let w = self.parse_expr()?;
            self.expect(TokenType::Comma, "Expected ',' between size dimensions")?;
            let h = self.parse_expr()?;
            self.expect(TokenType::Rparen, "Expected ')' after size")?;
            (None, Some(w), Some(h))
        };
        self.expect(TokenType::Semicolon, "Expected ';' after zone declaration")?;
        Ok(SafetyZoneDecl::SafetyZoneDecl {
            name: name.lexeme,
            shape,
            x,
            y,
            radius,
            width,
            height,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_max_speed_rule(&mut self) -> Result<SafetyRule, SpandaError> {
        let start = self.advance();
        let name = start.lexeme.clone();
        self.expect(TokenType::Assign, "Expected '=' in safety rule")?;
        let value = self.parse_expr()?;
        let unit = if let Expr::UnitLiteralExpr { unit, .. } = &value {
            *unit
        } else {
            self.parse_unit_suffix()?
        };
        self.expect(TokenType::Semicolon, "Expected ';' after safety rule")?;
        Ok(SafetyRule::MaxSpeedRule {
            name,
            value,
            unit,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_stop_if_rule(&mut self) -> Result<SafetyRule, SpandaError> {
        let start = self.advance();
        let condition = self.parse_expr()?;
        self.expect(TokenType::Semicolon, "Expected ';' after stop_if rule")?;
        Ok(SafetyRule::StopIfRule {
            condition,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_contract_clauses(
        &mut self,
    ) -> Result<(Option<Expr>, Option<Expr>, Option<Expr>), SpandaError> {
        let mut requires = None;
        let mut ensures = None;
        let mut invariant = None;
        while !self.check(TokenType::Lbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Requires]) {
                requires = Some(self.parse_expr()?);
            } else if self.match_types(&[TokenType::Ensures]) {
                ensures = Some(self.parse_expr()?);
            } else if self.match_types(&[TokenType::Invariant]) {
                invariant = Some(self.parse_expr()?);
            } else {
                break;
            }
        }
        Ok((requires, ensures, invariant))
    }

    fn parse_behavior(&mut self) -> Result<BehaviorDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected behavior name")?;
        self.expect(TokenType::Lparen, "Expected '(' after behavior name")?;
        self.expect(TokenType::Rparen, "Expected ')' after behavior parameters")?;
        let (requires, ensures, invariant) = self.parse_contract_clauses()?;
        self.expect(TokenType::Lbrace, "Expected '{' after behavior signature")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close behavior")?;
        Ok(BehaviorDecl::BehaviorDecl {
            name: name.lexeme,
            requires,
            ensures,
            invariant,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_task(&mut self) -> Result<TaskDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected task name")?;
        self.expect(TokenType::Every, "Expected 'every' after task name")?;
        let interval_ms = self.parse_duration()?;
        let (requires, ensures, invariant) = self.parse_contract_clauses()?;
        self.expect(TokenType::Lbrace, "Expected '{' after task signature")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close task")?;
        Ok(TaskDecl::TaskDecl {
            name: name.lexeme,
            interval_ms,
            requires,
            ensures,
            invariant,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_state_machine(&mut self) -> Result<StateMachineDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected state machine name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after state machine name")?;
        let mut states = Vec::new();
        let mut transitions = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::State]) {
                states.push(self.expect(TokenType::Ident, "Expected state name")?.lexeme);
                self.expect(TokenType::Semicolon, "Expected ';' after state")?;
            } else if self.match_types(&[TokenType::Transition]) {
                let from = self.expect(TokenType::Ident, "Expected source state")?;
                self.expect(TokenType::Arrow, "Expected '->' in transition")?;
                let to = self.expect(TokenType::Ident, "Expected target state")?;
                self.expect(TokenType::Semicolon, "Expected ';' after transition")?;
                let from_name = from.lexeme.clone();
                let to_name = to.lexeme;
                transitions.push(TransitionDecl {
                    from: from_name,
                    to: to_name,
                    span: self.span_from(&from, self.previous()),
                });
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected state or transition in state machine".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close state machine")?;
        Ok(StateMachineDecl::StateMachineDecl {
            name: name.lexeme,
            states,
            transitions,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_event(&mut self) -> Result<EventDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected event name")?;
        self.expect(TokenType::Semicolon, "Expected ';' after event")?;
        Ok(EventDecl::EventDecl {
            name: name.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_event_handler(&mut self) -> Result<EventHandlerDecl, SpandaError> {
        let start = self.advance(); // on
        let event_name = self.expect(TokenType::Ident, "Expected event name after on")?;
        self.expect(TokenType::Lbrace, "Expected '{' after event handler")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close event handler")?;
        Ok(EventHandlerDecl::EventHandlerDecl {
            event_name: event_name.lexeme,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_twin(&mut self) -> Result<TwinDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected twin name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after twin name")?;
        let mut mirrors = Vec::new();
        let mut replay = false;
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Mirror]) {
                mirrors.push(self.expect(TokenType::Ident, "Expected mirror field")?.lexeme);
                self.expect(TokenType::Semicolon, "Expected ';' after mirror")?;
            } else if self.match_types(&[TokenType::Replay]) {
                replay = self.match_types(&[TokenType::True]);
                if !replay {
                    self.expect(TokenType::False, "Expected true or false after replay")?;
                }
                self.expect(TokenType::Semicolon, "Expected ';' after replay")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected mirror or replay in twin block".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close twin")?;
        Ok(TwinDecl::TwinDecl {
            name: name.lexeme,
            mirrors,
            replay,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, SpandaError> {
        let mut stmts = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, SpandaError> {
        let start = self.peek().clone();
        if self.match_types(&[TokenType::Let]) {
            let name = self.parse_local_name("Expected variable name")?;
            self.expect(TokenType::Assign, "Expected '=' in let declaration")?;
            let init = self.parse_expr()?;
            self.expect(TokenType::Semicolon, "Expected ';' after let declaration")?;
            return Ok(Stmt::VarDecl {
                name: name.lexeme,
                init,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::If]) {
            let condition = self.parse_expr()?;
            self.expect(TokenType::Lbrace, "Expected '{' after if condition")?;
            let then_branch = self.parse_block()?;
            self.expect(TokenType::Rbrace, "Expected '}' after if block")?;
            let else_branch = if self.match_types(&[TokenType::Else]) {
                self.expect(TokenType::Lbrace, "Expected '{' after else")?;
                let branch = self.parse_block()?;
                self.expect(TokenType::Rbrace, "Expected '}' after else block")?;
                Some(branch)
            } else {
                None
            };
            return Ok(Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Loop]) {
            self.expect(TokenType::Every, "Expected 'every' after loop")?;
            let interval_ms = self.parse_duration()?;
            self.expect(TokenType::Lbrace, "Expected '{' after loop interval")?;
            let body = self.parse_block()?;
            let end = self.expect(TokenType::Rbrace, "Expected '}' to close loop")?;
            return Ok(Stmt::LoopStmt {
                interval_ms,
                body,
                span: self.span_from(&start, &end),
            });
        }
        if self.match_types(&[TokenType::Publish]) {
            let topic = self.expect(TokenType::Ident, "Expected topic name after publish")?;
            self.expect(TokenType::With, "Expected 'with' after topic name")?;
            let value = self.parse_expr()?;
            self.expect(TokenType::Semicolon, "Expected ';' after publish statement")?;
            return Ok(Stmt::PublishStmt {
                topic_name: topic.lexeme,
                value,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Call]) {
            let service = self.expect(TokenType::Ident, "Expected service name after call")?;
            self.expect(TokenType::Lparen, "Expected '(' after service name")?;
            self.expect(TokenType::Rparen, "Expected ')' after service arguments")?;
            self.expect(TokenType::Semicolon, "Expected ';' after service call")?;
            return Ok(Stmt::ServiceCallStmt {
                service_name: service.lexeme,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::SendGoal]) {
            let action = self.expect(TokenType::Ident, "Expected action name after send_goal")?;
            self.expect(TokenType::With, "Expected 'with' after action name")?;
            let goal = self.parse_expr()?;
            self.expect(TokenType::Semicolon, "Expected ';' after send_goal statement")?;
            return Ok(Stmt::ActionSendStmt {
                action_name: action.lexeme,
                goal,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::EmergencyStop]) {
            self.expect(TokenType::Semicolon, "Expected ';' after emergency_stop")?;
            return Ok(Stmt::EmergencyStopStmt {
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::ResetEmergencyStop]) {
            self.expect(TokenType::Semicolon, "Expected ';' after reset_emergency_stop")?;
            return Ok(Stmt::ResetEmergencyStopStmt {
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Emit]) {
            let event = self.parse_label("Expected event name after emit")?;
            self.expect(TokenType::Semicolon, "Expected ';' after emit statement")?;
            return Ok(Stmt::EmitStmt {
                event_name: event,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Enter]) {
            let state = self.parse_label("Expected state name after enter")?;
            self.expect(TokenType::Semicolon, "Expected ';' after enter statement")?;
            return Ok(Stmt::EnterStmt {
                state_name: state,
                span: self.span_from(&start, self.previous()),
            });
        }
        let expr = self.parse_expr()?;
        self.expect(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::ExprStmt {
            expr,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_duration(&mut self) -> Result<f64, SpandaError> {
        let tok = self.peek().clone();
        if tok.token_type == TokenType::UnitLiteral && tok.unit == Some(UnitLexeme::Ms) {
            self.advance();
            return Ok(num(&tok));
        }
        if tok.token_type == TokenType::UnitLiteral && tok.unit == Some(UnitLexeme::S) {
            self.advance();
            return Ok(num(&tok) * 1000.0);
        }
        if tok.token_type == TokenType::Number {
            self.advance();
            if self.check(TokenType::Ident) && self.peek().lexeme == "ms" {
                self.advance();
            }
            return Ok(num(&tok));
        }
        Err(SpandaError::Parse {
            message: "Expected duration like 50ms".into(),
            line: tok.line,
            column: tok.column,
        })
    }

    fn parse_unit_suffix(&mut self) -> Result<UnitKind, SpandaError> {
        self.try_parse_unit_suffix().ok_or_else(|| {
            let t = self.peek();
            SpandaError::Parse {
                message: "Expected unit suffix".into(),
                line: t.line,
                column: t.column,
            }
        })
    }

    fn try_parse_unit_suffix(&mut self) -> Option<UnitKind> {
        if self.check(TokenType::UnitLiteral) {
            let t = self.advance();
            return Some(unit_from_lexeme(t.unit?));
        }
        if self.check(TokenType::Ident) && self.peek().lexeme == "m"
            && self.tokens.get(self.pos + 1).map(|t| t.token_type) == Some(TokenType::Slash)
            && self.tokens.get(self.pos + 2).map(|t| t.lexeme.as_str()) == Some("s")
        {
            self.pos += 3;
            return Some(UnitKind::MPerS);
        }
        if self.check(TokenType::Ident) && self.peek().lexeme == "rad"
            && self.tokens.get(self.pos + 1).map(|t| t.token_type) == Some(TokenType::Slash)
            && self.tokens.get(self.pos + 2).map(|t| t.lexeme.as_str()) == Some("s")
        {
            self.pos += 3;
            return Some(UnitKind::RadPerS);
        }
        if self.check(TokenType::Ident) {
            let lexeme = self.peek().lexeme.clone();
            if is_unit_ident(&lexeme) {
                self.advance();
                return Some(UnitKind::from_lexeme(&lexeme));
            }
        }
        None
    }

    fn parse_expr(&mut self) -> Result<Expr, SpandaError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, SpandaError> {
        let mut left = self.parse_and()?;
        while self.match_types(&[TokenType::Or]) {
            let op_start = self.previous().clone();
            let right = self.parse_and()?;
            left = Expr::BinaryExpr {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(&op_start, self.previous()),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, SpandaError> {
        let mut left = self.parse_comparison()?;
        while self.match_types(&[TokenType::And]) {
            let op_start = self.previous().clone();
            let right = self.parse_comparison()?;
            left = Expr::BinaryExpr {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(&op_start, self.previous()),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, SpandaError> {
        let mut left = self.parse_additive()?;
        while self.match_types(&[
            TokenType::Lt,
            TokenType::Lte,
            TokenType::Gt,
            TokenType::Gte,
            TokenType::Eq,
            TokenType::Neq,
        ]) {
            let op_tok = self.previous().clone();
            let right = self.parse_additive()?;
            left = Expr::BinaryExpr {
                op: BinaryOp::from_lexeme(&op_tok.lexeme).unwrap(),
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(&op_tok, self.previous()),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, SpandaError> {
        let mut left = self.parse_multiplicative()?;
        while self.match_types(&[TokenType::Plus, TokenType::Minus]) {
            let op_tok = self.previous().clone();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryExpr {
                op: BinaryOp::from_lexeme(&op_tok.lexeme).unwrap(),
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(&op_tok, self.previous()),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, SpandaError> {
        let mut left = self.parse_unary()?;
        while self.match_types(&[TokenType::Star, TokenType::Slash]) {
            let op_tok = self.previous().clone();
            let right = self.parse_unary()?;
            left = Expr::BinaryExpr {
                op: BinaryOp::from_lexeme(&op_tok.lexeme).unwrap(),
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(&op_tok, self.previous()),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, SpandaError> {
        if self.match_types(&[TokenType::Minus, TokenType::Not]) {
            let op_tok = self.previous().clone();
            let op = if op_tok.token_type == TokenType::Not {
                UnaryOp::Not
            } else {
                UnaryOp::Neg
            };
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryExpr {
                op,
                operand: Box::new(operand),
                span: self.span_from(&op_tok, self.previous()),
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, SpandaError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.match_types(&[TokenType::Dot]) {
                let prop = self.parse_property_name()?;
                let start = expr_span(&expr);
                expr = Expr::MemberExpr {
                    object: Box::new(expr),
                    property: prop.lexeme,
                    span: Span {
                        start: start.start,
                        end: loc(self.previous()),
                    },
                };
            } else if self.match_types(&[TokenType::Lparen]) {
                let mut args = Vec::new();
                let mut named_args = Vec::new();
                if !self.check(TokenType::Rparen) {
                    loop {
                        if self.is_named_arg_start() {
                            let name = self.parse_named_arg_name()?;
                            self.advance();
                            named_args.push(NamedArg {
                                name,
                                value: self.parse_expr()?,
                                span: self.span_from(self.previous(), self.previous()),
                            });
                        } else {
                            args.push(self.parse_expr()?);
                        }
                        if !self.match_types(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                let end = self.expect(TokenType::Rparen, "Expected ')' after arguments")?;
                let start = expr_span(&expr);
                expr = Expr::CallExpr {
                    callee: Box::new(expr),
                    args,
                    named_args,
                    span: Span {
                        start: start.start,
                        end: loc(&end),
                    },
                };
            } else if self.check(TokenType::Lbrace) {
                if let Expr::IdentExpr { name, .. } = &expr {
                    if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        self.advance();
                        let mut fields = Vec::new();
                        if !self.check(TokenType::Rbrace) {
                            loop {
                                let fstart = self.peek().clone();
                                let field_name = self.parse_label("Expected struct field name")?;
                                self.expect(TokenType::Colon, "Expected ':' after struct field name")?;
                                let value = self.parse_expr()?;
                                fields.push(StructFieldInit {
                                    name: field_name,
                                    value,
                                    span: self.span_from(&fstart, self.previous()),
                                });
                                if !self.match_types(&[TokenType::Comma]) {
                                    break;
                                }
                            }
                        }
                        let end = self.expect(TokenType::Rbrace, "Expected '}' to close struct literal")?;
                        let start = expr_span(&expr);
                        expr = Expr::StructLiteralExpr {
                            type_name: name.clone(),
                            fields,
                            span: Span {
                                start: start.start,
                                end: loc(&end),
                            },
                        };
                        continue;
                    }
                }
                break;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, SpandaError> {
        let start = self.peek().clone();
        if self.match_types(&[TokenType::Match]) {
            let scrutinee = self.parse_expr()?;
            self.expect(TokenType::Lbrace, "Expected '{' after match scrutinee")?;
            let mut arms = Vec::new();
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                let arm_start = self.peek().clone();
                let variant = self.parse_label("Expected match arm variant")?;
                self.expect(TokenType::FatArrow, "Expected '=>' in match arm")?;
                let body = if self.check(TokenType::Lbrace) {
                    self.advance();
                    let stmts = self.parse_block()?;
                    self.expect(TokenType::Rbrace, "Expected '}' to close match arm")?;
                    stmts
                } else {
                    vec![self.parse_stmt()?]
                };
                arms.push(MatchArm {
                    variant,
                    body,
                    span: self.span_from(&arm_start, self.previous()),
                });
            }
            let end = self.expect(TokenType::Rbrace, "Expected '}' to close match")?;
            return Ok(Expr::MatchExpr {
                scrutinee: Box::new(scrutinee),
                arms,
                span: self.span_from(&start, &end),
            });
        }
        if self.match_types(&[TokenType::Robot]) {
            return Ok(Expr::IdentExpr {
                name: "robot".into(),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Safety]) {
            return Ok(Expr::IdentExpr {
                name: "safety".into(),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::True]) {
            return Ok(Expr::LiteralExpr {
                value: LiteralValue::Bool(true),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::False]) {
            return Ok(Expr::LiteralExpr {
                value: LiteralValue::Bool(false),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Number]) {
            let tok = self.previous().clone();
            if let Some(unit) = self.try_parse_unit_suffix() {
                return Ok(Expr::UnitLiteralExpr {
                    value: num(&tok),
                    unit,
                    span: self.span_from(&start, self.previous()),
                });
            }
            return Ok(Expr::LiteralExpr {
                value: LiteralValue::Number(num(&tok)),
                span: self.span_from(&start, &tok),
            });
        }
        if self.match_types(&[TokenType::UnitLiteral]) {
            let tok = self.previous().clone();
            return Ok(Expr::UnitLiteralExpr {
                value: num(&tok),
                unit: unit_from_lexeme(tok.unit.unwrap()),
                span: self.span_from(&start, &tok),
            });
        }
        if self.match_types(&[TokenType::String]) {
            return Ok(Expr::LiteralExpr {
                value: LiteralValue::String(str_val(self.previous())),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[
            TokenType::Ident,
            TokenType::Action,
            TokenType::State,
            TokenType::Plan,
            TokenType::Goal,
            TokenType::Skill,
            TokenType::Event,
            TokenType::Task,
            TokenType::Twin,
            TokenType::Match,
        ]) {
            let tok = self.previous();
            return Ok(Expr::IdentExpr {
                name: tok.lexeme.clone(),
                span: self.span_from(&start, tok),
            });
        }
        if self.match_types(&[TokenType::Lparen]) {
            let mut expr = self.parse_expr()?;
            let end = self.expect(TokenType::Rparen, "Expected ')' after expression")?;
            let _old = expr_span(&expr);
            expr = re_span_expr(expr, Span {
                start: loc(&start),
                end: loc(&end),
            });
            return Ok(expr);
        }
        let t = self.peek();
        Err(SpandaError::Parse {
            message: "Expected expression".into(),
            line: t.line,
            column: t.column,
        })
    }

    fn parse_property_name(&mut self) -> Result<Token, SpandaError> {
        let lexeme = self.parse_label("Expected property name after '.'")?;
        Ok(Token {
            token_type: TokenType::Ident,
            lexeme,
            value: TokenValue::Null,
            unit: None,
            line: self.previous().line,
            column: self.previous().column,
            offset: self.previous().offset,
        })
    }

    fn parse_local_name(&mut self, message: &str) -> Result<Token, SpandaError> {
        let lexeme = self.parse_label(message)?;
        Ok(Token {
            token_type: TokenType::Ident,
            lexeme,
            value: TokenValue::Null,
            unit: None,
            line: self.previous().line,
            column: self.previous().column,
            offset: self.previous().offset,
        })
    }

    fn is_named_arg_start(&self) -> bool {
        self.tokens.get(self.pos + 1).map(|t| t.token_type) == Some(TokenType::Colon)
            && (self.check(TokenType::Ident) || self.check(TokenType::From))
    }

    fn parse_named_arg_name(&mut self) -> Result<String, SpandaError> {
        if self.match_types(&[TokenType::From]) {
            Ok("from".into())
        } else {
            Ok(self.advance().lexeme)
        }
    }
}

fn loc(t: &Token) -> SourceLocation {
    SourceLocation {
        line: t.line,
        column: t.column,
        offset: t.offset,
    }
}

fn num(tok: &Token) -> f64 {
    match &tok.value {
        TokenValue::Number(n) => *n,
        _ => 0.0,
    }
}

fn str_val(tok: &Token) -> String {
    match &tok.value {
        TokenValue::String(s) => s.clone(),
        _ => tok.lexeme.clone(),
    }
}

fn is_unit_ident(lexeme: &str) -> bool {
    matches!(lexeme, "m" | "s" | "ms" | "rad" | "deg" | "Hz")
}

fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::LiteralExpr { span, .. }
        | Expr::UnitLiteralExpr { span, .. }
        | Expr::IdentExpr { span, .. }
        | Expr::BinaryExpr { span, .. }
        | Expr::UnaryExpr { span, .. }
        | Expr::CallExpr { span, .. }
        | Expr::MemberExpr { span, .. }
        | Expr::MatchExpr { span, .. }
        | Expr::StructLiteralExpr { span, .. } => *span,
    }
}

fn re_span_expr(expr: Expr, span: Span) -> Expr {
    match expr {
        Expr::LiteralExpr { value, .. } => Expr::LiteralExpr { value, span },
        Expr::UnitLiteralExpr { value, unit, .. } => Expr::UnitLiteralExpr { value, unit, span },
        Expr::IdentExpr { name, .. } => Expr::IdentExpr { name, span },
        Expr::BinaryExpr { op, left, right, .. } => Expr::BinaryExpr { op, left, right, span },
        Expr::UnaryExpr { op, operand, .. } => Expr::UnaryExpr { op, operand, span },
        Expr::CallExpr {
            callee,
            args,
            named_args,
            ..
        } => Expr::CallExpr {
            callee,
            args,
            named_args,
            span,
        },
        Expr::MemberExpr { object, property, .. } => Expr::MemberExpr {
            object,
            property,
            span,
        },
        Expr::MatchExpr { scrutinee, arms, .. } => Expr::MatchExpr {
            scrutinee,
            arms,
            span,
        },
        Expr::StructLiteralExpr {
            type_name,
            fields,
            ..
        } => Expr::StructLiteralExpr {
            type_name,
            fields,
            span,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn parses_complete_robot() {
        let source = r#"
robot Rover {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  safety {
    max_speed = 1.5 m/s;
    stop_if lidar.read().nearest_distance < 0.5 m;
  }
  behavior avoid_obstacles() {
    loop every 50ms {
      let scan = lidar.read();
      if scan.nearest_distance < 0.5 m {
        wheels.stop();
      } else {
        wheels.drive(linear: 0.8 m/s, angular: 0.2 rad/s);
      }
    }
  }
}
"#;
        let ast = parse(tokenize(source).unwrap()).unwrap();
        assert_eq!(ast.robots().len(), 1);
        let RobotDecl::RobotDecl {
            name,
            sensors,
            actuators,
            safety,
            behaviors,
            ..
        } = &ast.robots()[0];
        assert_eq!(name, "Rover");
        assert_eq!(sensors.len(), 1);
        let SensorDecl::SensorDecl {
            name,
            sensor_type,
            binding,
            ..
        } = &sensors[0];
        assert_eq!(name, "lidar");
        assert_eq!(sensor_type, "Lidar");
        assert_eq!(binding, &Some(SensorBinding::Topic { path: "/scan".into() }));
        assert_eq!(actuators.len(), 1);
        assert!(safety.is_some());
        assert_eq!(behaviors.len(), 1);
    }

    #[test]
    fn parses_loop_every() {
        let ast = parse(tokenize("robot R { behavior b() { loop every 50ms { } } }").unwrap()).unwrap();
        let RobotDecl::RobotDecl { behaviors, .. } = &ast.robots()[0];
        let BehaviorDecl::BehaviorDecl { body, .. } = &behaviors[0];
        match &body[0] {
            Stmt::LoopStmt { interval_ms, .. } => assert_eq!(*interval_ms, 50.0),
            _ => panic!("expected loop"),
        }
    }

    #[test]
    fn parses_if_else() {
        let ast = parse(tokenize("robot R { behavior b() { if true { } else { } } }").unwrap()).unwrap();
        let RobotDecl::RobotDecl { behaviors, .. } = &ast.robots()[0];
        let BehaviorDecl::BehaviorDecl { body, .. } = &behaviors[0];
        match &body[0] {
            Stmt::IfStmt { else_branch, .. } => assert!(else_branch.is_some()),
            _ => panic!("expected if"),
        }
    }

    #[test]
    fn parses_max_speed_rule() {
        let ast = parse(tokenize("robot R { safety { max_speed = 1.5 m/s; } }").unwrap()).unwrap();
        let RobotDecl::RobotDecl { safety, .. } = &ast.robots()[0];
        let Some(s) = safety else {
            panic!("expected safety block");
        };
        let SafetyBlock::SafetyBlock { rules, .. } = s;
        assert!(matches!(rules[0], SafetyRule::MaxSpeedRule { .. }));
    }
}
