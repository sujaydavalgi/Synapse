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

type ContractClauses = (Option<Expr>, Option<Expr>, Option<Expr>);

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

    fn parse_binding_ident(&mut self, message: &str) -> Result<String, SpandaError> {
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
                | TokenType::Mission
                | TokenType::Duration
                | TokenType::Network
                | TokenType::Bandwidth
                | TokenType::Latency
                | TokenType::Timing
                | TokenType::Budget
                | TokenType::Fault
                | TokenType::Execute
                | TokenType::Discover
                | TokenType::Subscribe
                | TokenType::Receive
                | TokenType::Message
                | TokenType::Response
                | TokenType::Feedback
                | TokenType::Result
                | TokenType::Request
                | TokenType::Device
                | TokenType::Bus
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
                | TokenType::Execute
                | TokenType::Discover
                | TokenType::Subscribe
                | TokenType::Receive
                | TokenType::Message
                | TokenType::Response
                | TokenType::Feedback
                | TokenType::Result
                | TokenType::Request
                | TokenType::Device
                | TokenType::Bus
                | TokenType::Qos
                | TokenType::Reliable
                | TokenType::BestEffort
                | TokenType::Rate
                | TokenType::History
                | TokenType::Deadline
                | TokenType::Telemetry
                | TokenType::Faults
                | TokenType::Verify
                | TokenType::Requires
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

    fn parse_hal_binding_name(&mut self, message: &str) -> Result<String, SpandaError> {
        let t = self.peek();
        let ok = matches!(
            t.token_type,
            TokenType::Ident
                | TokenType::Battery
                | TokenType::Memory
                | TokenType::Cpu
                | TokenType::Storage
                | TokenType::Gpu
                | TokenType::Sensors
                | TokenType::Actuators
                | TokenType::Capacity
                | TokenType::Hardware
                | TokenType::Deploy
                | TokenType::Bus
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

    fn parse_type_name_part(&mut self, message: &str) -> Result<String, SpandaError> {
        if self.check(TokenType::Ident) {
            return Ok(self.advance().lexeme);
        }
        self.parse_label(message)
    }

    fn parse_type_annotation(&mut self) -> Result<SpandaType, SpandaError> {
        use crate::type_system::{resolve_generic_type, resolve_type_name};
        let start = self.peek().clone();
        let mut parts = vec![self.parse_type_name_part("Expected type name")?];
        while self.match_types(&[TokenType::Dot]) {
            parts.push(self.parse_type_name_part("Expected type name after '.'")?);
        }
        let qualified = parts.join(".");
        if self.match_types(&[TokenType::Lt]) {
            let mut args = Vec::new();
            if !self.check(TokenType::Gt) {
                loop {
                    args.push(self.parse_type_annotation()?);
                    if !self.match_types(&[TokenType::Comma]) {
                        break;
                    }
                }
            }
            self.expect(TokenType::Gt, "Expected '>' to close generic type")?;
            let base = parts.last().cloned().unwrap_or_default();
            resolve_generic_type(&base, &args).map_err(|msg| SpandaError::Parse {
                message: msg,
                line: start.line,
                column: start.column,
            })
        } else {
            match resolve_type_name(&qualified) {
                Ok(ty) => Ok(ty),
                Err(_msg) if !qualified.contains('.') => Ok(SpandaType::Named { name: qualified }),
                Err(msg) => Err(SpandaError::Parse {
                    message: msg,
                    line: start.line,
                    column: start.column,
                }),
            }
        }
    }

    fn parse_program(&mut self) -> Result<Program, SpandaError> {
        let start = self.peek().clone();
        let mut module_name = None;
        if self.check(TokenType::Module) {
            self.advance();
            module_name = Some(self.parse_dotted_name("Expected module name after 'module'")?);
            self.expect(
                TokenType::Semicolon,
                "Expected ';' after module declaration",
            )?;
        }
        let mut imports = Vec::new();
        let mut functions = Vec::new();
        let mut tests = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut hardware_profiles = Vec::new();
        let mut deployments = Vec::new();
        let mut requires_hardware = None;
        let mut requires_network = None;
        let mut simulate_compatibility = None;
        let mut messages = Vec::new();
        let mut robots = Vec::new();
        while self.check(TokenType::Import) {
            imports.push(self.parse_import()?);
        }
        while !self.check(TokenType::Eof) {
            if self.is_module_fn_start() {
                functions.push(self.parse_module_fn()?);
            } else if self.check(TokenType::Ident) && self.peek().lexeme == "test" {
                tests.push(self.parse_test()?);
            } else if self.check(TokenType::Struct) {
                structs.push(self.parse_struct()?);
            } else if self.check(TokenType::Enum) {
                enums.push(self.parse_enum()?);
            } else if self.check(TokenType::Trait) {
                traits.push(self.parse_trait()?);
            } else if self.check(TokenType::Hardware) {
                hardware_profiles.push(self.parse_hardware()?);
            } else if self.check(TokenType::Deploy) {
                deployments.push(self.parse_deploy()?);
            } else if self.check(TokenType::RequiresHardware) {
                requires_hardware = Some(self.parse_requires_hardware()?);
            } else if self.check(TokenType::RequiresNetwork) {
                requires_network = Some(self.parse_requires_network()?);
            } else if self.check(TokenType::SimulateCompatibility) {
                simulate_compatibility = Some(self.parse_simulate_compatibility()?);
            } else if self.check(TokenType::Message) {
                messages.push(self.parse_message()?);
            } else if self.check(TokenType::Robot) {
                robots.push(self.parse_robot()?);
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected struct, enum, trait, hardware, deploy, or robot declaration"
                        .into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        Ok(Program::Program {
            module_name,
            imports,
            functions,
            tests,
            structs,
            enums,
            traits,
            hardware_profiles,
            deployments,
            requires_hardware,
            requires_network,
            simulate_compatibility,
            messages,
            robots,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_hardware(&mut self) -> Result<HardwareDecl, SpandaError> {
        use crate::foundations::HardwareDecl;
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected hardware profile name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after hardware name")?;
        let mut cpu = None;
        let mut memory_mb = None;
        let mut storage_mb = None;
        let mut gpu_tops = None;
        let mut gpu_required = false;
        let mut sensors = Vec::new();
        let mut actuators = Vec::new();
        let mut battery_wh = None;
        let mut network_bandwidth_mbps = None;
        let mut network_latency_ms = None;
        let mut min_control_period_ms = None;
        let mut power_draw_w = None;

        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Cpu]) {
                self.expect(TokenType::Colon, "Expected ':' after cpu")?;
                cpu = Some(self.parse_label("Expected CPU identifier")?);
                self.expect(TokenType::Semicolon, "Expected ';' after cpu")?;
            } else if self.match_types(&[TokenType::Memory]) {
                self.expect(TokenType::Colon, "Expected ':' after memory")?;
                memory_mb = Some(self.parse_storage_amount()?);
                self.expect(TokenType::Semicolon, "Expected ';' after memory")?;
            } else if self.match_types(&[TokenType::Storage]) {
                self.expect(TokenType::Colon, "Expected ':' after storage")?;
                storage_mb = Some(self.parse_storage_amount()?);
                self.expect(TokenType::Semicolon, "Expected ';' after storage")?;
            } else if self.match_types(&[TokenType::Gpu]) {
                self.expect(TokenType::Colon, "Expected ':' after gpu")?;
                if self.check(TokenType::True) {
                    self.advance();
                    gpu_required = true;
                } else {
                    gpu_tops = Some(self.parse_number_value()?);
                    if self.check(TokenType::Ident) && self.peek().lexeme == "TOPS" {
                        self.advance();
                    }
                }
                self.expect(TokenType::Semicolon, "Expected ';' after gpu")?;
            } else if self.match_types(&[TokenType::Sensors]) {
                sensors = self.parse_hardware_type_list("sensors")?;
            } else if self.match_types(&[TokenType::Actuators]) {
                actuators = self.parse_hardware_type_list("actuators")?;
            } else if self.match_types(&[TokenType::Battery]) {
                self.expect(TokenType::Lbrace, "Expected '{' after battery")?;
                while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                    if self.match_types(&[TokenType::Capacity]) {
                        self.expect(TokenType::Colon, "Expected ':' after capacity")?;
                        battery_wh = Some(self.parse_energy_wh_value()?);
                        self.expect(TokenType::Semicolon, "Expected ';' after capacity")?;
                    } else {
                        let t = self.peek();
                        return Err(SpandaError::Parse {
                            message: "Expected capacity in battery block".into(),
                            line: t.line,
                            column: t.column,
                        });
                    }
                }
                self.expect(TokenType::Rbrace, "Expected '}' to close battery block")?;
            } else if self.match_types(&[TokenType::Network]) {
                self.expect(TokenType::Lbrace, "Expected '{' after network")?;
                while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                    if self.match_types(&[TokenType::Bandwidth]) {
                        self.expect(TokenType::Colon, "Expected ':' after bandwidth")?;
                        network_bandwidth_mbps = Some(self.parse_network_amount()?);
                        self.expect(TokenType::Semicolon, "Expected ';' after bandwidth")?;
                    } else if self.match_types(&[TokenType::Latency]) {
                        self.expect(TokenType::Colon, "Expected ':' after latency")?;
                        network_latency_ms = Some(self.parse_duration()?);
                        self.expect(TokenType::Semicolon, "Expected ';' after latency")?;
                    } else {
                        let t = self.peek();
                        return Err(SpandaError::Parse {
                            message: "Expected bandwidth or latency in network block".into(),
                            line: t.line,
                            column: t.column,
                        });
                    }
                }
                self.expect(TokenType::Rbrace, "Expected '}' to close network block")?;
            } else if self.match_types(&[TokenType::Timing]) {
                self.expect(TokenType::Lbrace, "Expected '{' after timing")?;
                while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                    if self.match_types(&[TokenType::MinPeriod]) {
                        self.expect(TokenType::Colon, "Expected ':' after min_period")?;
                        min_control_period_ms = Some(self.parse_duration()?);
                        self.expect(TokenType::Semicolon, "Expected ';' after min_period")?;
                    } else {
                        let t = self.peek();
                        return Err(SpandaError::Parse {
                            message: "Expected min_period in timing block".into(),
                            line: t.line,
                            column: t.column,
                        });
                    }
                }
                self.expect(TokenType::Rbrace, "Expected '}' to close timing block")?;
            } else if self.match_types(&[TokenType::Resource]) {
                self.expect(TokenType::Colon, "Expected ':' after resource")?;
                power_draw_w = Some(self.parse_number_value()?);
                if self.check(TokenType::Ident) && self.peek().lexeme == "W" {
                    self.advance();
                }
                self.expect(TokenType::Semicolon, "Expected ';' after resource power")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected hardware profile member".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close hardware block")?;
        Ok(HardwareDecl::HardwareDecl {
            name: name.lexeme,
            cpu,
            memory_mb,
            storage_mb,
            gpu_tops,
            gpu_required,
            sensors,
            actuators,
            battery_wh,
            network_bandwidth_mbps,
            network_latency_ms,
            min_control_period_ms,
            power_draw_w,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_hardware_type_list(&mut self, kind: &str) -> Result<Vec<String>, SpandaError> {
        self.expect(TokenType::Lbracket, &format!("Expected '[' after {kind}"))?;
        let mut items = Vec::new();
        if !self.check(TokenType::Rbracket) {
            loop {
                items.push(self.parse_label(&format!("Expected {kind} type name"))?);
                if self.match_types(&[TokenType::Comma]) {
                    if self.check(TokenType::Rbracket) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }
        self.expect(
            TokenType::Rbracket,
            &format!("Expected ']' after {kind} list"),
        )?;
        self.expect(
            TokenType::Semicolon,
            &format!("Expected ';' after {kind} list"),
        )?;
        Ok(items)
    }

    fn parse_storage_amount(&mut self) -> Result<f64, SpandaError> {
        let value = self.parse_number_value()?;
        if self.check(TokenType::Ident) {
            let unit = self.peek().lexeme.as_str();
            let mb = match unit {
                "GB" | "Gb" => value * 1024.0,
                "MB" | "Mb" => value,
                "TB" | "Tb" => value * 1024.0 * 1024.0,
                _ => value,
            };
            if unit == "GB"
                || unit == "MB"
                || unit == "TB"
                || unit == "Gb"
                || unit == "Mb"
                || unit == "Tb"
            {
                self.advance();
            }
            return Ok(mb);
        }
        Ok(value)
    }

    fn parse_number_value(&mut self) -> Result<f64, SpandaError> {
        let tok = self.expect(TokenType::Number, "Expected number")?;
        match tok.value {
            TokenValue::Number(n) => Ok(n),
            _ => Err(SpandaError::Parse {
                message: "Expected number".into(),
                line: tok.line,
                column: tok.column,
            }),
        }
    }

    fn parse_deploy(&mut self) -> Result<DeployDecl, SpandaError> {
        use crate::foundations::DeployDecl;
        let start = self.advance();
        let robot_name = self.parse_label("Expected robot name after deploy")?;
        self.expect(TokenType::To, "Expected 'to' after deploy robot name")?;
        let mut targets = Vec::new();
        if self.match_types(&[TokenType::Lbracket]) {
            if !self.check(TokenType::Rbracket) {
                loop {
                    targets.push(self.parse_label("Expected hardware target name")?);
                    if !self.match_types(&[TokenType::Comma]) {
                        break;
                    }
                }
            }
            self.expect(TokenType::Rbracket, "Expected ']' after deploy targets")?;
        } else {
            targets.push(self.parse_label("Expected hardware target name")?);
        }
        self.expect(TokenType::Semicolon, "Expected ';' after deploy statement")?;
        let end = self.previous();
        Ok(DeployDecl::DeployDecl {
            robot_name,
            targets,
            span: self.span_from(&start, end),
        })
    }

    fn parse_network_amount(&mut self) -> Result<f64, SpandaError> {
        let value = self.parse_number_value()?;
        if self.check(TokenType::Ident) {
            let unit = self.peek().lexeme.as_str();
            if unit == "Mbps" || unit == "mbps" {
                self.advance();
                return Ok(value);
            }
            if unit == "Gbps" || unit == "gbps" {
                self.advance();
                return Ok(value * 1000.0);
            }
        }
        Ok(value)
    }

    fn parse_requires_hardware(
        &mut self,
    ) -> Result<crate::foundations::RequiresHardwareDecl, SpandaError> {
        use crate::foundations::RequiresHardwareDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after requires_hardware")?;
        let mut memory_mb_min = None;
        let mut storage_mb_min = None;
        let mut gpu_tops_min = None;
        let mut gpu_required = false;
        let mut sensors = Vec::new();
        let mut actuators = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Memory]) {
                self.expect(
                    TokenType::Gte,
                    "Expected '>=' after memory in requires_hardware",
                )?;
                memory_mb_min = Some(self.parse_storage_amount()?);
                self.expect(
                    TokenType::Semicolon,
                    "Expected ';' after memory requirement",
                )?;
            } else if self.match_types(&[TokenType::Storage]) {
                self.expect(
                    TokenType::Gte,
                    "Expected '>=' after storage in requires_hardware",
                )?;
                storage_mb_min = Some(self.parse_storage_amount()?);
                self.expect(
                    TokenType::Semicolon,
                    "Expected ';' after storage requirement",
                )?;
            } else if self.match_types(&[TokenType::Gpu]) {
                if self.check(TokenType::Gte) {
                    self.advance();
                    gpu_tops_min = Some(self.parse_number_value()?);
                    if self.check(TokenType::Ident) && self.peek().lexeme == "TOPS" {
                        self.advance();
                    }
                } else {
                    self.expect(TokenType::Colon, "Expected ':' or '>=' after gpu")?;
                    if self.match_types(&[TokenType::True]) {
                        gpu_required = true;
                    } else {
                        gpu_tops_min = Some(self.parse_number_value()?);
                        if self.check(TokenType::Ident) && self.peek().lexeme == "TOPS" {
                            self.advance();
                        }
                    }
                }
                self.expect(TokenType::Semicolon, "Expected ';' after gpu requirement")?;
            } else if self.match_types(&[TokenType::Sensors]) {
                sensors = self.parse_hardware_type_list("sensors")?;
            } else if self.match_types(&[TokenType::Actuators]) {
                actuators = self.parse_hardware_type_list("actuators")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected requires_hardware member".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close requires_hardware")?;
        Ok(RequiresHardwareDecl::RequiresHardwareDecl {
            memory_mb_min,
            storage_mb_min,
            gpu_tops_min,
            gpu_required,
            sensors,
            actuators,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_requires_network(
        &mut self,
    ) -> Result<crate::foundations::RequiresNetworkDecl, SpandaError> {
        use crate::foundations::RequiresNetworkDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after requires_network")?;
        let mut bandwidth_mbps_min = None;
        let mut latency_ms_max = None;
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Bandwidth]) {
                self.expect(TokenType::Gte, "Expected '>=' after bandwidth")?;
                bandwidth_mbps_min = Some(self.parse_network_amount()?);
                self.expect(
                    TokenType::Semicolon,
                    "Expected ';' after bandwidth requirement",
                )?;
            } else if self.match_types(&[TokenType::Latency]) {
                self.expect(TokenType::Lte, "Expected '<=' after latency")?;
                latency_ms_max = Some(self.parse_duration()?);
                self.expect(
                    TokenType::Semicolon,
                    "Expected ';' after latency requirement",
                )?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected bandwidth or latency in requires_network".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close requires_network")?;
        Ok(RequiresNetworkDecl::RequiresNetworkDecl {
            bandwidth_mbps_min,
            latency_ms_max,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_simulate_compatibility(
        &mut self,
    ) -> Result<crate::foundations::SimulateCompatibilityDecl, SpandaError> {
        use crate::foundations::{SimFaultDecl, SimulateCompatibilityDecl};
        let start = self.advance();
        self.expect(
            TokenType::Lbrace,
            "Expected '{' after simulate_compatibility",
        )?;
        let mut faults = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Fault]) {
                let fault_start = self.peek().clone();
                let fault_type = self.parse_label("Expected fault type name")?;
                self.expect(TokenType::Semicolon, "Expected ';' after fault")?;
                faults.push(SimFaultDecl {
                    fault_type,
                    span: self.span_from(&fault_start, self.previous()),
                });
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected fault declaration in simulate_compatibility".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(
            TokenType::Rbrace,
            "Expected '}' to close simulate_compatibility",
        )?;
        Ok(SimulateCompatibilityDecl::SimulateCompatibilityDecl {
            faults,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_mission(&mut self) -> Result<crate::foundations::MissionDecl, SpandaError> {
        use crate::foundations::MissionDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after mission")?;
        let mut duration_hours = None;
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Duration]) {
                self.expect(TokenType::Colon, "Expected ':' after duration")?;
                duration_hours = Some(self.parse_duration_hours()?);
                self.expect(TokenType::Semicolon, "Expected ';' after duration")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected duration in mission block".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close mission")?;
        let duration_hours = duration_hours.ok_or_else(|| {
            let t = self.peek();
            SpandaError::Parse {
                message: "mission block requires duration".into(),
                line: t.line,
                column: t.column,
            }
        })?;
        Ok(MissionDecl::MissionDecl {
            duration_hours,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_duration_hours(&mut self) -> Result<f64, SpandaError> {
        if self.check(TokenType::UnitLiteral) {
            let tok = self.advance();
            if let (TokenValue::Number(n), Some(unit_lex)) = (tok.value, tok.unit) {
                return Ok(Self::duration_to_hours(n, unit_from_lexeme(unit_lex)));
            }
        }
        let value = self.parse_number_value()?;
        if self.check(TokenType::Ident) {
            let unit = self.peek().lexeme.as_str();
            let hours = match unit {
                "h" | "hr" | "hrs" | "hour" | "hours" => value,
                "min" | "mins" | "minute" | "minutes" => value / 60.0,
                "s" | "sec" | "secs" => value / 3600.0,
                _ => value,
            };
            if unit == "h"
                || unit == "hr"
                || unit == "hrs"
                || unit == "hour"
                || unit == "hours"
                || unit == "min"
                || unit == "mins"
                || unit == "minute"
                || unit == "minutes"
                || unit == "s"
                || unit == "sec"
                || unit == "secs"
            {
                self.advance();
            }
            return Ok(hours);
        }
        Ok(value)
    }

    fn duration_to_hours(value: f64, unit: UnitKind) -> f64 {
        match unit {
            UnitKind::H => value,
            UnitKind::Min => value / 60.0,
            UnitKind::S => value / 3600.0,
            UnitKind::Ms => value / 3_600_000.0,
            UnitKind::Us => value / 3_600_000_000.0,
            _ => value,
        }
    }

    fn parse_energy_wh_value(&mut self) -> Result<f64, SpandaError> {
        if self.check(TokenType::UnitLiteral) {
            let tok = self.advance();
            if let (TokenValue::Number(n), Some(unit_lex)) = (tok.value, tok.unit) {
                let unit = unit_from_lexeme(unit_lex);
                return Ok(match unit {
                    UnitKind::Wh => n,
                    UnitKind::KWh => n * 1000.0,
                    UnitKind::Joule => n / 3600.0,
                    _ => n,
                });
            }
        }
        let value = self.parse_number_value()?;
        if self.check(TokenType::Ident) {
            let unit = self.peek().lexeme.as_str();
            let wh = match unit {
                "Wh" => value,
                "kWh" => value * 1000.0,
                "J" => value / 3600.0,
                _ => value,
            };
            if unit == "Wh" || unit == "kWh" || unit == "J" {
                self.advance();
            }
            return Ok(wh);
        }
        Ok(value)
    }

    fn parse_budget(&mut self) -> Result<crate::foundations::ResourceBudgetDecl, SpandaError> {
        use crate::foundations::ResourceBudgetDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after budget")?;
        let mut battery_pct_max = None;
        let mut memory_mb_max = None;
        let mut cpu_pct_max = None;
        let mut network_mbps_max = None;
        let mut storage_mb_max = None;
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Battery]) {
                self.expect(TokenType::Lte, "Expected '<=' after battery in budget")?;
                battery_pct_max = Some(self.parse_percent_value()?);
                self.expect(TokenType::Semicolon, "Expected ';' after battery budget")?;
            } else if self.match_types(&[TokenType::Memory]) {
                self.expect(TokenType::Lte, "Expected '<=' after memory in budget")?;
                memory_mb_max = Some(self.parse_storage_amount()?);
                self.expect(TokenType::Semicolon, "Expected ';' after memory budget")?;
            } else if self.match_types(&[TokenType::Cpu]) {
                self.expect(TokenType::Lte, "Expected '<=' after cpu in budget")?;
                cpu_pct_max = Some(self.parse_percent_value()?);
                self.expect(TokenType::Semicolon, "Expected ';' after cpu budget")?;
            } else if self.match_types(&[TokenType::Network]) {
                self.expect(TokenType::Lte, "Expected '<=' after network in budget")?;
                network_mbps_max = Some(self.parse_network_amount()?);
                self.expect(TokenType::Semicolon, "Expected ';' after network budget")?;
            } else if self.match_types(&[TokenType::Storage]) {
                self.expect(TokenType::Lte, "Expected '<=' after storage in budget")?;
                storage_mb_max = Some(self.parse_storage_amount()?);
                self.expect(TokenType::Semicolon, "Expected ';' after storage budget")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected budget constraint".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close budget")?;
        Ok(ResourceBudgetDecl::ResourceBudgetDecl {
            battery_pct_max,
            memory_mb_max,
            cpu_pct_max,
            network_mbps_max,
            storage_mb_max,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_percent_value(&mut self) -> Result<f64, SpandaError> {
        let value = self.parse_number_value()?;
        if self.check(TokenType::Ident) && self.peek().lexeme == "%" {
            self.advance();
        } else if self.match_types(&[TokenType::Percent]) {
            // consumed
        }
        Ok(value)
    }

    fn parse_dotted_name(&mut self, message: &str) -> Result<String, SpandaError> {
        let mut parts = vec![self.parse_import_segment(message)?];
        while self.match_types(&[TokenType::Dot]) {
            parts.push(self.parse_import_segment("Expected name after '.'")?);
        }
        Ok(parts.join("."))
    }

    fn parse_import(&mut self) -> Result<ImportDecl, SpandaError> {
        let start = self.advance();
        let vendor = self.parse_import_segment("Expected library vendor name")?;
        self.expect(TokenType::Dot, "Expected '.' in import path")?;
        let module = self.parse_import_segment("Expected library module name")?;
        self.expect(TokenType::Semicolon, "Expected ';' after import")?;
        Ok(ImportDecl::ImportDecl {
            path: format!("{}.{}", vendor, module),
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_import_segment(&mut self, message: &str) -> Result<String, SpandaError> {
        if self.check(TokenType::Ident) {
            return Ok(self.advance().lexeme);
        }
        if self.check(TokenType::Eof)
            || self.check(TokenType::Dot)
            || self.check(TokenType::Semicolon)
        {
            let t = self.peek();
            return Err(SpandaError::Parse {
                message: message.to_string(),
                line: t.line,
                column: t.column,
            });
        }
        Ok(self.advance().lexeme)
    }

    fn is_module_fn_start(&self) -> bool {
        self.check(TokenType::Export)
            || self.check(TokenType::Public)
            || self.check(TokenType::Private)
            || self.check(TokenType::Async)
            || self.check(TokenType::Fn)
    }

    fn parse_type_params(&mut self) -> Result<Vec<String>, SpandaError> {
        if !self.match_types(&[TokenType::Lt]) {
            return Ok(Vec::new());
        }
        let mut params = Vec::new();
        loop {
            params.push(self.parse_label("Expected type parameter name")?);
            if !self.match_types(&[TokenType::Comma]) {
                break;
            }
        }
        self.expect(TokenType::Gt, "Expected '>' after type parameters")?;
        Ok(params)
    }

    fn parse_module_fn(&mut self) -> Result<crate::foundations::ModuleFnDecl, SpandaError> {
        use crate::foundations::{ModuleFnDecl, ModuleParamDecl, Visibility};
        let start = self.peek().clone();
        let mut visibility = Visibility::Private;
        if self.match_types(&[TokenType::Export]) {
            visibility = Visibility::Export;
        } else if self.match_types(&[TokenType::Public]) {
            visibility = Visibility::Public;
        } else if self.match_types(&[TokenType::Private]) {
            visibility = Visibility::Private;
        }
        let is_async = self.match_types(&[TokenType::Async]);
        self.expect(TokenType::Fn, "Expected 'fn' in module function")?;
        let name = self.parse_label("Expected function name")?;
        let type_params = self.parse_type_params()?;
        self.expect(TokenType::Lparen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(TokenType::Rparen) {
            loop {
                let pstart = self.peek().clone();
                let pname = self.parse_label("Expected parameter name")?;
                self.expect(TokenType::Colon, "Expected ':' after parameter name")?;
                let type_ann = self.parse_type_annotation()?;
                params.push(ModuleParamDecl {
                    name: pname,
                    type_ann,
                    span: self.span_from(&pstart, self.previous()),
                });
                if !self.match_types(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.expect(TokenType::Rparen, "Expected ')' after parameters")?;
        self.expect(TokenType::Arrow, "Expected '->' after function parameters")?;
        let return_type = self.parse_type_annotation()?;
        self.expect(TokenType::Lbrace, "Expected '{' after function return type")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close function")?;
        Ok(ModuleFnDecl {
            name,
            visibility,
            type_params,
            params,
            return_type,
            is_async,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_test(&mut self) -> Result<crate::foundations::TestDecl, SpandaError> {
        use crate::foundations::TestDecl;
        let start = self.advance(); // test ident
        let name_tok = self.expect(TokenType::String, "Expected test name string")?;
        let TokenValue::String(name) = name_tok.value else {
            return Err(SpandaError::Parse {
                message: "Expected test name string".into(),
                line: name_tok.line,
                column: name_tok.column,
            });
        };
        self.expect(TokenType::Lbrace, "Expected '{' after test name")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close test")?;
        Ok(TestDecl {
            name,
            body,
            span: self.span_from(&start, &end),
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
            variants.push(
                self.expect(TokenType::Ident, "Expected enum variant")?
                    .lexeme,
            );
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
        self.expect(
            TokenType::Arrow,
            "Expected '->' after trait method parameters",
        )?;
        let return_type = self.expect(TokenType::Ident, "Expected return type")?;
        self.expect(TokenType::Semicolon, "Expected ';' after trait method")?;
        Ok(TraitMethodDecl {
            name,
            params,
            return_type: return_type.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn is_robot_member_keyword(&self, kw: &str) -> bool {
        self.check(TokenType::Ident) && self.peek().lexeme == kw
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
        let mut verify = None;
        let mut observe = None;
        let mut identity = None;
        let mut audit = None;
        let mut provenance = None;
        let mut signed_records = Vec::new();
        let mut secrets = Vec::new();
        let mut trust = None;
        let mut permissions = None;
        let mut trait_impls = Vec::new();
        let mut requires_hardware = None;
        let mut requires_network = None;
        let mut mission = None;
        let mut buses = Vec::new();
        let mut peer_robots = Vec::new();
        let mut devices = Vec::new();
        let mut agent_channels = Vec::new();
        let twin_sync = None;
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
            } else if self.check(TokenType::Ident) && self.is_agent_channel() {
                agent_channels.push(self.parse_agent_channel()?);
            } else if self.check(TokenType::Agent) {
                if self.is_agent_shorthand() {
                    self.parse_agent_shorthand(&mut agents)?;
                } else {
                    agents.push(self.parse_agent()?);
                }
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
            } else if self.check(TokenType::Verify) {
                verify = Some(self.parse_verify()?);
            } else if self.check(TokenType::Observe) {
                observe = Some(self.parse_observe()?);
            } else if self.is_robot_member_keyword("identity") {
                identity = Some(self.parse_identity()?);
            } else if self.is_robot_member_keyword("audit") {
                audit = Some(self.parse_audit()?);
            } else if self.is_robot_member_keyword("provenance") {
                provenance = Some(self.parse_provenance()?);
            } else if self.is_robot_member_keyword("record") {
                signed_records.push(self.parse_signed_record()?);
            } else if self.check(TokenType::Secret) {
                secrets.push(self.parse_secret()?);
            } else if self.check(TokenType::Trust) {
                trust = Some(self.parse_trust()?);
            } else if self.check(TokenType::Permissions) {
                permissions = Some(self.parse_permissions()?);
            } else if self.check(TokenType::RequiresHardware) {
                requires_hardware = Some(self.parse_requires_hardware()?);
            } else if self.check(TokenType::RequiresNetwork) {
                requires_network = Some(self.parse_requires_network()?);
            } else if self.check(TokenType::Mission) {
                mission = Some(self.parse_mission()?);
            } else if self.check(TokenType::Impl) {
                trait_impls.push(self.parse_trait_impl()?);
            } else if self.check(TokenType::Bus) {
                buses.push(self.parse_bus()?);
            } else if self.check(TokenType::Robot) {
                peer_robots.push(self.parse_peer_robot()?);
            } else if self.check(TokenType::Device) {
                devices.push(self.parse_device()?);
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
            verify,
            observe,
            identity,
            audit,
            provenance,
            signed_records,
            secrets,
            trust,
            permissions,
            requires_hardware,
            requires_network,
            mission,
            trait_impls,
            buses,
            peer_robots,
            devices,
            agent_channels,
            twin_sync,
            span: self.span_from(&start, &end),
        })
    }

    fn is_agent_shorthand(&mut self) -> bool {
        let mut idx = self.pos + 1;
        if idx >= self.tokens.len() {
            return false;
        }
        if self.tokens[idx].token_type != TokenType::Ident {
            return false;
        }
        idx += 1;
        idx < self.tokens.len() && self.tokens[idx].token_type == TokenType::Semicolon
    }

    fn is_agent_channel(&self) -> bool {
        let idx = self.pos;
        idx + 2 < self.tokens.len()
            && self.tokens[idx].token_type == TokenType::Ident
            && self.tokens[idx + 1].token_type == TokenType::Arrow
            && self.tokens[idx + 2].token_type == TokenType::Ident
    }

    fn parse_agent_shorthand(&mut self, agents: &mut Vec<AgentDecl>) -> Result<(), SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected agent name")?;
        self.expect(TokenType::Semicolon, "Expected ';' after agent reference")?;
        agents.push(AgentDecl::AgentDecl {
            name: name.lexeme,
            uses_ai: Vec::new(),
            memory_kind: None,
            tools: Vec::new(),
            skills: Vec::new(),
            capabilities: Vec::new(),
            goal: String::new(),
            plan_body: Vec::new(),
            span: self.span_from(&start, self.previous()),
        });
        Ok(())
    }

    fn parse_bus(&mut self) -> Result<crate::comm::BusDecl, SpandaError> {
        use crate::comm::{BusDecl, TransportKind};
        let start = self.advance();
        let transport_name = self.expect(TokenType::Ident, "Expected bus transport name")?;
        self.expect(TokenType::Semicolon, "Expected ';' after bus declaration")?;
        let transport =
            TransportKind::from_ident(&transport_name.lexeme).unwrap_or(TransportKind::Local);
        Ok(BusDecl::BusDecl {
            name: transport_name.lexeme.clone(),
            transport,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_peer_robot(&mut self) -> Result<crate::comm::PeerRobotDecl, SpandaError> {
        use crate::comm::PeerRobotDecl;
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected peer robot name")?;
        self.expect(TokenType::Semicolon, "Expected ';' after peer robot")?;
        Ok(PeerRobotDecl::PeerRobotDecl {
            name: name.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_device(&mut self) -> Result<crate::comm::DeviceDecl, SpandaError> {
        use crate::comm::DeviceDecl;
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected device name")?;
        self.expect(TokenType::Colon, "Expected ':' after device name")?;
        let device_type = self.expect(TokenType::Ident, "Expected device type")?;
        self.expect(
            TokenType::Semicolon,
            "Expected ';' after device declaration",
        )?;
        Ok(DeviceDecl::DeviceDecl {
            name: name.lexeme,
            device_type: device_type.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_agent_channel(&mut self) -> Result<crate::comm::AgentChannelDecl, SpandaError> {
        use crate::comm::AgentChannelDecl;
        let start = self.peek().clone();
        let from_agent = self
            .expect(TokenType::Ident, "Expected source agent")?
            .lexeme;
        self.expect(TokenType::Arrow, "Expected '->' in agent channel")?;
        let to_agent = self
            .expect(TokenType::Ident, "Expected target agent")?
            .lexeme;
        self.expect(TokenType::Semicolon, "Expected ';' after agent channel")?;
        Ok(AgentChannelDecl::AgentChannelDecl {
            from_agent,
            to_agent,
            message_type: String::new(),
            span: self.span_from(&start, self.previous()),
        })
    }

    #[allow(dead_code)]
    fn parse_twin_sync(&mut self) -> Result<crate::comm::TwinSyncDecl, SpandaError> {
        use crate::comm::TwinSyncDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after twin sync")?;
        let mut telemetry = false;
        let mut replay = false;
        let mut faults = false;
        let mut events = false;
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Telemetry]) {
                telemetry = true;
                self.expect(TokenType::Semicolon, "Expected ';' after telemetry")?;
            } else if self.match_types(&[TokenType::Replay]) {
                replay = true;
                self.expect(TokenType::Semicolon, "Expected ';' after replay")?;
            } else if self.match_types(&[TokenType::Faults]) {
                faults = true;
                self.expect(TokenType::Semicolon, "Expected ';' after faults")?;
            } else if self.match_types(&[TokenType::Event]) {
                events = true;
                self.expect(TokenType::Semicolon, "Expected ';' after events")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected telemetry, replay, faults, or events in twin sync".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close twin sync")?;
        Ok(TwinSyncDecl::TwinSyncDecl {
            telemetry,
            replay,
            faults,
            events,
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

    fn parse_trait_impl_method(
        &mut self,
    ) -> Result<crate::foundations::TraitImplMethodDecl, SpandaError> {
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
                let ptype = self
                    .expect(TokenType::Ident, "Expected parameter type")?
                    .lexeme;
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
        self.expect(
            TokenType::Arrow,
            "Expected '->' after trait impl parameters",
        )?;
        let return_type = self
            .expect(TokenType::Ident, "Expected return type")?
            .lexeme;
        self.expect(
            TokenType::Lbrace,
            "Expected '{' after trait impl method signature",
        )?;
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
            let name = self.parse_hal_binding_name("Expected I2C bus name")?;
            self.expect(TokenType::At, "Expected 'at' after I2C bus name")?;
            let addr = self.expect(TokenType::Number, "Expected I2C address")?;
            self.expect(TokenType::Semicolon, "Expected ';' after I2C declaration")?;
            return Ok(HalMemberDecl::HalI2cDecl {
                name,
                address: num(&addr),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Spi]) {
            let name = self.parse_hal_binding_name("Expected SPI bus name")?;
            self.expect(TokenType::At, "Expected 'at' after SPI bus name")?;
            let bus = self.expect(TokenType::Number, "Expected SPI bus number")?;
            let mut cs_pin = None;
            if self.match_types(&[TokenType::Pin]) {
                cs_pin = Some(num(
                    &self.expect(TokenType::Number, "Expected CS pin number")?
                ));
            }
            self.expect(TokenType::Semicolon, "Expected ';' after SPI declaration")?;
            return Ok(HalMemberDecl::HalSpiDecl {
                name,
                bus: num(&bus),
                cs_pin,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Gpio]) {
            let name = self.parse_hal_binding_name("Expected GPIO name")?;
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
                name,
                direction,
                pin: num(&pin),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Pwm]) {
            let name = self.parse_hal_binding_name("Expected PWM name")?;
            self.expect(TokenType::On, "Expected 'on' after PWM name")?;
            self.expect(TokenType::Pin, "Expected 'pin' after on")?;
            let pin = self.expect(TokenType::Number, "Expected PWM pin")?;
            self.expect(TokenType::Frequency, "Expected 'frequency' after PWM pin")?;
            let freq = self.parse_frequency_hz()?;
            self.expect(TokenType::Semicolon, "Expected ';' after PWM declaration")?;
            return Ok(HalMemberDecl::HalPwmDecl {
                name,
                pin: num(&pin),
                frequency_hz: freq,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Uart]) {
            let name = self.parse_hal_binding_name("Expected UART name")?;
            self.expect(TokenType::On, "Expected 'on' after UART name")?;
            let device = self.expect(TokenType::String, "Expected UART device path")?;
            self.expect(TokenType::Baud, "Expected 'baud' after UART device")?;
            let baud = self.expect(TokenType::Number, "Expected baud rate")?;
            self.expect(TokenType::Semicolon, "Expected ';' after UART declaration")?;
            return Ok(HalMemberDecl::HalUartDecl {
                name,
                device: str_val(&device),
                baud: num(&baud),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Adc]) {
            let name = self.parse_hal_binding_name("Expected ADC name")?;
            self.expect(TokenType::On, "Expected 'on' after ADC name")?;
            self.expect(TokenType::Ident, "Expected 'channel' keyword")?;
            let ch = self.expect(TokenType::Number, "Expected ADC channel number")?;
            self.expect(TokenType::Semicolon, "Expected ';' after ADC declaration")?;
            return Ok(HalMemberDecl::HalAdcDecl {
                name,
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

    fn parse_message(&mut self) -> Result<crate::comm::MessageDecl, SpandaError> {
        use crate::comm::MessageDecl;
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected message name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after message name")?;
        let mut fields = Vec::new();
        let mut version = None;
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.check(TokenType::Ident) && self.peek().lexeme == "version" {
                self.advance();
                self.expect(TokenType::Colon, "Expected ':' after version")?;
                version = Some(self.parse_number_value()? as u32);
                self.expect(TokenType::Semicolon, "Expected ';' after version")?;
                continue;
            }
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
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close message")?;
        Ok(MessageDecl::MessageDecl {
            name: name.lexeme,
            fields,
            version,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_qos_block(&mut self) -> Result<crate::comm::QosDecl, SpandaError> {
        use crate::comm::{QosDecl, QosReliability};
        let start = self.peek().clone();
        self.expect(TokenType::Lbrace, "Expected '{' for topic QoS block")?;
        let mut reliability = None;
        let mut rate_hz = None;
        let mut deadline_ms = None;
        let mut history = None;
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Qos]) {
                if self.match_types(&[TokenType::Reliable]) {
                    reliability = Some(QosReliability::Reliable);
                } else if self.match_types(&[TokenType::BestEffort]) {
                    reliability = Some(QosReliability::BestEffort);
                }
                self.expect(TokenType::Semicolon, "Expected ';' after qos reliability")?;
            } else if self.match_types(&[TokenType::Rate]) {
                rate_hz = Some(self.parse_frequency_hz()?);
                self.expect(TokenType::Semicolon, "Expected ';' after rate")?;
            } else if self.match_types(&[TokenType::Deadline]) {
                deadline_ms = Some(self.parse_duration()?);
                self.expect(TokenType::Semicolon, "Expected ';' after deadline")?;
            } else if self.match_types(&[TokenType::History]) {
                history = Some(
                    self.expect(TokenType::Ident, "Expected history policy")?
                        .lexeme,
                );
                self.expect(TokenType::Semicolon, "Expected ';' after history")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected qos, rate, deadline, or history in topic block".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close QoS block")?;
        Ok(QosDecl {
            reliability,
            rate_hz,
            deadline_ms,
            history,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_node(&mut self) -> Result<NodeDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected node name")?;
        let namespace = if self.match_types(&[TokenType::On]) {
            Some(str_val(&self.expect(
                TokenType::String,
                "Expected namespace string after 'on'",
            )?))
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
        use crate::comm::{TopicRole, TransportKind};
        let start = self.advance();
        let name = self.parse_label("Expected topic name")?;
        self.expect(TokenType::Colon, "Expected ':' after topic name")?;
        let message_type = self.parse_label("Expected message type")?;

        let mut role = TopicRole::Both;
        let mut topic_path = None;
        let mut qos = None;
        let mut transport = None;

        if self.match_types(&[TokenType::Publish]) {
            role = TopicRole::Publish;
            if self.match_types(&[TokenType::On]) {
                if self.check(TokenType::String) {
                    topic_path = Some(str_val(&self.advance()));
                } else {
                    let ident =
                        self.expect(TokenType::Ident, "Expected transport or topic path")?;
                    transport = TransportKind::from_ident(&ident.lexeme);
                }
            }
        } else if self.match_types(&[TokenType::Subscribe]) {
            role = TopicRole::Subscribe;
            if self.match_types(&[TokenType::On]) {
                if self.check(TokenType::String) {
                    topic_path = Some(str_val(&self.advance()));
                } else {
                    let ident =
                        self.expect(TokenType::Ident, "Expected transport or topic path")?;
                    transport = TransportKind::from_ident(&ident.lexeme);
                }
            }
        }

        if self.check(TokenType::Lbrace) {
            qos = Some(self.parse_qos_block()?);
        }

        if self.match_types(&[TokenType::On]) && topic_path.is_none() && transport.is_none() {
            if self.check(TokenType::String) {
                topic_path = Some(str_val(&self.advance()));
            } else {
                let ident = self.expect(TokenType::Ident, "Expected transport name after on")?;
                transport = TransportKind::from_ident(&ident.lexeme);
            }
        }

        let secure = if self.check(TokenType::Secure) {
            Some(self.parse_secure_block()?)
        } else {
            None
        };

        self.expect(TokenType::Semicolon, "Expected ';' after topic declaration")?;
        Ok(TopicDecl::TopicDecl {
            name,
            message_type,
            topic: topic_path,
            role,
            qos,
            transport,
            secure,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_service(&mut self) -> Result<ServiceDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected service name")?;

        if self.check(TokenType::Lbrace) {
            self.advance();
            let mut request_type = None;
            let mut response_type = None;
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                if self.match_types(&[TokenType::Request]) {
                    request_type = Some(
                        self.expect(TokenType::Ident, "Expected request type")?
                            .lexeme,
                    );
                    self.expect(TokenType::Semicolon, "Expected ';' after request type")?;
                } else if self.match_types(&[TokenType::Response]) {
                    response_type = Some(
                        self.expect(TokenType::Ident, "Expected response type")?
                            .lexeme,
                    );
                    self.expect(TokenType::Semicolon, "Expected ';' after response type")?;
                } else {
                    let t = self.peek();
                    return Err(SpandaError::Parse {
                        message: "Expected request or response in service block".into(),
                        line: t.line,
                        column: t.column,
                    });
                }
            }
            self.expect(TokenType::Rbrace, "Expected '}' to close service")?;
            let secure = if self.check(TokenType::Secure) {
                Some(self.parse_secure_block()?)
            } else {
                None
            };
            self.expect(
                TokenType::Semicolon,
                "Expected ';' after service declaration",
            )?;
            return Ok(ServiceDecl::ServiceDecl {
                name: name.lexeme,
                service_type: None,
                request_type,
                response_type,
                secure,
                span: self.span_from(&start, self.previous()),
            });
        }

        self.expect(TokenType::Colon, "Expected ':' after service name")?;
        let service_type = self.expect(TokenType::Ident, "Expected service type")?;
        let secure = if self.check(TokenType::Secure) {
            Some(self.parse_secure_block()?)
        } else {
            None
        };
        self.expect(
            TokenType::Semicolon,
            "Expected ';' after service declaration",
        )?;
        Ok(ServiceDecl::ServiceDecl {
            name: name.lexeme,
            service_type: Some(service_type.lexeme),
            request_type: None,
            response_type: None,
            secure,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_action(&mut self) -> Result<ActionDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected action name")?;

        if self.check(TokenType::Lbrace) {
            self.advance();
            let mut request_type = None;
            let mut feedback_type = None;
            let mut result_type = None;
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                if self.match_types(&[TokenType::Request]) {
                    request_type = Some(
                        self.expect(TokenType::Ident, "Expected request type")?
                            .lexeme,
                    );
                    self.expect(TokenType::Semicolon, "Expected ';' after request type")?;
                } else if self.match_types(&[TokenType::Feedback]) {
                    feedback_type = Some(
                        self.expect(TokenType::Ident, "Expected feedback type")?
                            .lexeme,
                    );
                    self.expect(TokenType::Semicolon, "Expected ';' after feedback type")?;
                } else if self.match_types(&[TokenType::Result]) {
                    result_type = Some(
                        self.expect(TokenType::Ident, "Expected result type")?
                            .lexeme,
                    );
                    self.expect(TokenType::Semicolon, "Expected ';' after result type")?;
                } else {
                    let t = self.peek();
                    return Err(SpandaError::Parse {
                        message: "Expected request, feedback, or result in action block".into(),
                        line: t.line,
                        column: t.column,
                    });
                }
            }
            self.expect(TokenType::Rbrace, "Expected '}' to close action")?;
            let secure = if self.check(TokenType::Secure) {
                Some(self.parse_secure_block()?)
            } else {
                None
            };
            self.expect(
                TokenType::Semicolon,
                "Expected ';' after action declaration",
            )?;
            return Ok(ActionDecl::ActionDecl {
                name: name.lexeme,
                action_type: None,
                request_type,
                feedback_type,
                result_type,
                secure,
                span: self.span_from(&start, self.previous()),
            });
        }

        self.expect(TokenType::Colon, "Expected ':' after action name")?;
        let action_type = self.expect(TokenType::Ident, "Expected action type")?;
        let secure = if self.check(TokenType::Secure) {
            Some(self.parse_secure_block()?)
        } else {
            None
        };
        self.expect(
            TokenType::Semicolon,
            "Expected ';' after action declaration",
        )?;
        Ok(ActionDecl::ActionDecl {
            name: name.lexeme,
            action_type: Some(action_type.lexeme),
            request_type: None,
            feedback_type: None,
            result_type: None,
            secure,
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
                    bus_name: self.parse_hal_binding_name(
                        "Expected HAL bus name or topic string after 'on'",
                    )?,
                })
            }
        } else {
            None
        };
        self.expect(
            TokenType::Semicolon,
            "Expected ';' after sensor declaration",
        )?;
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
        self.expect(
            TokenType::Semicolon,
            "Expected ';' after actuator declaration",
        )?;
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

    fn parse_verify(&mut self) -> Result<crate::foundations::VerifyDecl, SpandaError> {
        use crate::foundations::VerifyDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after verify")?;
        let mut rules = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            rules.push(self.parse_expr()?);
            self.expect(TokenType::Semicolon, "Expected ';' after verify rule")?;
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close verify block")?;
        Ok(VerifyDecl::VerifyDecl {
            rules,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_observe(&mut self) -> Result<crate::foundations::ObserveDecl, SpandaError> {
        use crate::foundations::ObserveDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after observe")?;
        let mut sensors = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let sensor = self.expect(TokenType::Ident, "Expected sensor name in observe block")?;
            sensors.push(sensor.lexeme);
            self.expect(TokenType::Semicolon, "Expected ';' after observe sensor")?;
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close observe block")?;
        Ok(ObserveDecl::ObserveDecl {
            sensors,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_identity(&mut self) -> Result<IdentityDecl, SpandaError> {
        let start = self.advance();
        let type_name = self.expect(TokenType::Ident, "Expected identity type name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after identity type")?;
        let mut fields = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let key = self.parse_config_key_token()?;
            self.expect(TokenType::Colon, "Expected ':' in identity field")?;
            let value = self.parse_config_value_string()?;
            self.expect(TokenType::Semicolon, "Expected ';' after identity field")?;
            fields.push((key, value));
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close identity")?;
        Ok(IdentityDecl::IdentityDecl {
            type_name: type_name.lexeme,
            fields,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_audit(&mut self) -> Result<AuditDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected audit name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after audit name")?;
        let mut records = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            self.expect(TokenType::Ident, "Expected 'record' in audit block")?;
            if self.previous().lexeme != "record" {
                let t = self.previous();
                return Err(SpandaError::Parse {
                    message: "Expected 'record' in audit block".into(),
                    line: t.line,
                    column: t.column,
                });
            }
            records.push(self.parse_expr()?);
            self.expect(TokenType::Semicolon, "Expected ';' after audit record")?;
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close audit")?;
        Ok(AuditDecl::AuditDecl {
            name: name.lexeme,
            records,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_provenance(&mut self) -> Result<ProvenanceDecl, SpandaError> {
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected provenance name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after provenance name")?;
        let mut hash_algo = "sha256".to_string();
        let mut signed_by = String::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let key = self.parse_config_key_token()?;
            self.expect(TokenType::Colon, "Expected ':' in provenance field")?;
            match key.as_str() {
                "hash" => {
                    hash_algo = self
                        .expect(TokenType::Ident, "Expected hash algorithm name")?
                        .lexeme;
                }
                "signed_by" => {
                    signed_by = Self::expr_path_string(&self.parse_expr()?);
                }
                other => {
                    let t = self.peek();
                    return Err(SpandaError::Parse {
                        message: format!("Unknown provenance field '{other}'"),
                        line: t.line,
                        column: t.column,
                    });
                }
            }
            self.expect(TokenType::Semicolon, "Expected ';' after provenance field")?;
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close provenance")?;
        Ok(ProvenanceDecl::ProvenanceDecl {
            name: name.lexeme,
            hash_algo,
            signed_by,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_signed_record(&mut self) -> Result<SignedRecordDecl, SpandaError> {
        let start = self.advance();
        let event_name = self.expect(TokenType::Ident, "Expected signed record event name")?;
        self.expect(
            TokenType::SignedBy,
            "Expected 'signed_by' after record event",
        )?;
        let signed_by = Self::expr_path_string(&self.parse_expr()?);
        self.expect(
            TokenType::Semicolon,
            "Expected ';' after signed record declaration",
        )?;
        Ok(SignedRecordDecl::SignedRecordDecl {
            event_name: event_name.lexeme,
            signed_by,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_secret(&mut self) -> Result<crate::foundations::SecretDecl, SpandaError> {
        use crate::foundations::{SecretDecl, SecretSourceDecl};
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected secret name")?;
        self.expect(TokenType::From, "Expected 'from' after secret name")?;
        let source = if self.match_types(&[TokenType::Env]) {
            self.expect(TokenType::Lparen, "Expected '(' after env")?;
            let var = str_val(&self.expect(TokenType::String, "Expected env var name")?);
            self.expect(TokenType::Rparen, "Expected ')' after env var")?;
            SecretSourceDecl::Env { var }
        } else if self.check(TokenType::String) {
            SecretSourceDecl::Literal {
                value: str_val(&self.advance()),
            }
        } else {
            let t = self.peek();
            return Err(SpandaError::Parse {
                message: "Expected env(...) or string literal for secret source".into(),
                line: t.line,
                column: t.column,
            });
        };
        self.expect(
            TokenType::Semicolon,
            "Expected ';' after secret declaration",
        )?;
        Ok(SecretDecl::SecretDecl {
            name: name.lexeme,
            source,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_trust(&mut self) -> Result<crate::foundations::TrustDecl, SpandaError> {
        use crate::foundations::TrustDecl;
        let start = self.advance();
        let level = self
            .expect(TokenType::Ident, "Expected trust level")?
            .lexeme;
        self.expect(TokenType::Semicolon, "Expected ';' after trust declaration")?;
        Ok(TrustDecl::TrustDecl {
            level,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_dotted_capability(&mut self) -> Result<String, SpandaError> {
        let first = self.parse_label("Expected capability name")?;
        if self.match_types(&[TokenType::Dot]) {
            let second = self.parse_label("Expected capability suffix")?;
            Ok(format!("{first}.{second}"))
        } else {
            Ok(first)
        }
    }

    fn parse_permissions(&mut self) -> Result<crate::foundations::PermissionsDecl, SpandaError> {
        use crate::foundations::PermissionsDecl;
        let start = self.advance();
        self.expect(TokenType::Lbracket, "Expected '[' after permissions")?;
        let mut capabilities = Vec::new();
        if !self.check(TokenType::Rbracket) {
            loop {
                capabilities.push(self.parse_dotted_capability()?);
                if !self.match_types(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.expect(TokenType::Rbracket, "Expected ']' to close permissions")?;
        self.expect(
            TokenType::Semicolon,
            "Expected ';' after permissions declaration",
        )?;
        Ok(PermissionsDecl::PermissionsDecl {
            capabilities,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_secure_block(&mut self) -> Result<crate::foundations::SecureBlockDecl, SpandaError> {
        use crate::foundations::SecureBlockDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after secure")?;
        let mut signed = false;
        let mut min_trust = None;
        let mut requires = Vec::new();
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let field = self.parse_label("Expected secure field name")?;
            self.expect(TokenType::Assign, "Expected '=' in secure field")?;
            match field.as_str() {
                "signed" => {
                    signed = self.match_types(&[TokenType::True]);
                    if !signed {
                        self.expect(TokenType::False, "Expected true or false for signed")?;
                    }
                }
                "min_trust" => {
                    min_trust = Some(self.parse_label("Expected trust level")?);
                }
                "requires" => {
                    self.expect(TokenType::Lbracket, "Expected '[' after requires")?;
                    if !self.check(TokenType::Rbracket) {
                        loop {
                            requires.push(self.parse_dotted_capability()?);
                            if !self.match_types(&[TokenType::Comma]) {
                                break;
                            }
                        }
                    }
                    self.expect(TokenType::Rbracket, "Expected ']' after requires")?;
                }
                other => {
                    return Err(SpandaError::Parse {
                        message: format!("Unknown secure field '{other}'"),
                        line: self.previous().line,
                        column: self.previous().column,
                    });
                }
            }
            self.expect(TokenType::Semicolon, "Expected ';' after secure field")?;
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close secure block")?;
        Ok(SecureBlockDecl {
            signed,
            min_trust,
            requires,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_config_value_string(&mut self) -> Result<String, SpandaError> {
        let tok = self.advance();
        match tok.token_type {
            TokenType::String => Ok(tok.lexeme.trim_matches('"').to_string()),
            TokenType::Ident => Ok(tok.lexeme),
            _ => Err(SpandaError::Parse {
                message: "Expected string or identifier in config value".into(),
                line: tok.line,
                column: tok.column,
            }),
        }
    }

    fn expr_path_string(expr: &Expr) -> String {
        match expr {
            Expr::IdentExpr { name, .. } => name.clone(),
            Expr::MemberExpr {
                object, property, ..
            } => {
                format!("{}.{}", Self::expr_path_string(object), property)
            }
            _ => String::new(),
        }
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
            self.expect(
                TokenType::Semicolon,
                "Expected ';' after ai model config entry",
            )?;
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
        } else if self.check(TokenType::SignedBy) {
            self.advance();
            Ok("signed_by".into())
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
            let n = num(self.previous());
            if self.check(TokenType::Ident) {
                let unit = self.peek().lexeme.as_str();
                let scaled = match unit {
                    "GB" | "Gb" => n * 1024.0,
                    "MB" | "Mb" => n,
                    "TOPS" | "tops" => n,
                    _ => n,
                };
                if unit == "GB"
                    || unit == "Gb"
                    || unit == "MB"
                    || unit == "Mb"
                    || unit == "TOPS"
                    || unit == "tops"
                {
                    self.advance();
                }
                Ok(ConfigValue::Number(scaled))
            } else {
                Ok(ConfigValue::Number(n))
            }
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
                uses_ai.push(
                    self.expect(TokenType::Ident, "Expected model name after uses")?
                        .lexeme,
                );
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
        let action = if self.match_types(&[TokenType::Plan]) {
            "plan".to_string()
        } else if self.check(TokenType::Ident)
            || self.check(TokenType::Publish)
            || self.check(TokenType::Subscribe)
            || self.check(TokenType::Call)
            || self.check(TokenType::Execute)
            || self.check(TokenType::Discover)
        {
            self.advance().lexeme
        } else {
            return Err(SpandaError::Parse {
                message: "Expected capability action".into(),
                line: self.peek().line,
                column: self.peek().column,
            });
        };
        let target = if self.match_types(&[TokenType::Lparen]) {
            let t = self.expect(TokenType::Ident, "Expected capability target")?;
            self.expect(TokenType::Rparen, "Expected ')' after capability target")?;
            Some(t.lexeme)
        } else {
            None
        };
        Ok(CapabilityDecl {
            action,
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

    fn parse_contract_clauses(&mut self) -> Result<ContractClauses, SpandaError> {
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
        let mut budget = None;
        if self.check(TokenType::Budget) {
            budget = Some(self.parse_budget()?);
        }
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close task")?;
        Ok(TaskDecl::TaskDecl {
            name: name.lexeme,
            interval_ms,
            requires,
            ensures,
            invariant,
            budget,
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
        let mut fields = Vec::new();
        if self.check(TokenType::Lbrace) {
            self.advance();
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                let field_start = self.peek().clone();
                let field_name = self.expect(TokenType::Ident, "Expected event field name")?;
                self.expect(TokenType::Colon, "Expected ':' after event field name")?;
                let type_name = self.expect(TokenType::Ident, "Expected event field type")?;
                self.expect(TokenType::Semicolon, "Expected ';' after event field")?;
                fields.push(FieldDecl {
                    name: field_name.lexeme,
                    type_name: type_name.lexeme,
                    span: self.span_from(&field_start, self.previous()),
                });
            }
            self.expect(TokenType::Rbrace, "Expected '}' to close event")?;
        }
        self.expect(TokenType::Semicolon, "Expected ';' after event")?;
        Ok(EventDecl::EventDecl {
            name: name.lexeme,
            fields,
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
                mirrors.push(self.parse_label("Expected mirror field")?);
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
        if self.match_types(&[TokenType::Return]) {
            let value = if self.check(TokenType::Semicolon) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            self.expect(TokenType::Semicolon, "Expected ';' after return")?;
            return Ok(Stmt::ReturnStmt {
                value,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Let]) {
            let name = self.parse_local_name("Expected variable name")?;
            let type_annotation = if self.match_types(&[TokenType::Colon]) {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            let init = if self.match_types(&[TokenType::Assign]) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            if type_annotation.is_none() && init.is_none() {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected type annotation or initializer in let declaration".into(),
                    line: t.line,
                    column: t.column,
                });
            }
            self.expect(TokenType::Semicolon, "Expected ';' after let declaration")?;
            return Ok(Stmt::VarDecl {
                name: name.lexeme,
                type_annotation,
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
            let topic_name = self.parse_subscribe_target()?;
            if self.match_types(&[TokenType::Lparen]) {
                let value = self.parse_expr()?;
                self.expect(TokenType::Rparen, "Expected ')' after publish value")?;
                self.expect(TokenType::Semicolon, "Expected ';' after publish statement")?;
                return Ok(Stmt::PublishStmt {
                    topic_name,
                    value,
                    span: self.span_from(&start, self.previous()),
                });
            }
            self.expect(TokenType::With, "Expected 'with' or '(' after topic name")?;
            let value = self.parse_expr()?;
            self.expect(TokenType::Semicolon, "Expected ';' after publish statement")?;
            return Ok(Stmt::PublishStmt {
                topic_name,
                value,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Subscribe]) {
            let target = self.parse_subscribe_target()?;
            self.expect(TokenType::Semicolon, "Expected ';' after subscribe")?;
            return Ok(Stmt::SubscribeStmt {
                target,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Execute]) {
            let action = self.expect(TokenType::Ident, "Expected action name after execute")?;
            let goal = if self.match_types(&[TokenType::Lparen]) {
                let g = self.parse_expr()?;
                self.expect(TokenType::Rparen, "Expected ')' after execute goal")?;
                g
            } else {
                self.parse_expr()?
            };
            self.expect(TokenType::Semicolon, "Expected ';' after execute")?;
            return Ok(Stmt::ExecuteStmt {
                action_name: action.lexeme,
                goal,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Discover]) {
            let target = self.parse_discover_target()?;
            let filter = self.parse_discover_filter()?;
            self.expect(TokenType::Semicolon, "Expected ';' after discover")?;
            return Ok(Stmt::DiscoverStmt {
                target,
                filter,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Receive]) {
            let topic = self.expect(TokenType::Ident, "Expected topic name after receive")?;
            self.expect(TokenType::To, "Expected 'to' after topic in receive")?;
            let var = self.expect(TokenType::Ident, "Expected variable name")?;
            self.expect(TokenType::Semicolon, "Expected ';' after receive")?;
            return Ok(Stmt::ReceiveStmt {
                topic_name: topic.lexeme,
                var_name: var.lexeme,
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
            self.expect(
                TokenType::Semicolon,
                "Expected ';' after send_goal statement",
            )?;
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
            self.expect(
                TokenType::Semicolon,
                "Expected ';' after reset_emergency_stop",
            )?;
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
        if self.match_types(&[TokenType::Remember]) {
            let key = str_val(&self.expect(TokenType::String, "Expected memory key string")?);
            self.expect(TokenType::Comma, "Expected ',' after memory key")?;
            let value = self.parse_expr()?;
            self.expect(
                TokenType::Semicolon,
                "Expected ';' after remember statement",
            )?;
            return Ok(Stmt::RememberStmt {
                key,
                value,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Spawn]) {
            let callee = self.parse_label("Expected function name after spawn")?;
            let mut args = Vec::new();
            if self.match_types(&[TokenType::Lparen]) {
                if !self.check(TokenType::Rparen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.match_types(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.expect(TokenType::Rparen, "Expected ')' after spawn arguments")?;
            }
            self.expect(TokenType::Semicolon, "Expected ';' after spawn")?;
            return Ok(Stmt::SpawnStmt {
                callee: Expr::IdentExpr {
                    name: callee,
                    span: self.span_from(&start, self.previous()),
                },
                args,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Select]) {
            self.expect(TokenType::Lbrace, "Expected '{' after select")?;
            let mut arms = Vec::new();
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                let arm_start = self.peek().clone();
                let recv = self.expect(TokenType::Ident, "Expected 'recv' in select arm")?;
                if recv.lexeme != "recv" {
                    return Err(SpandaError::Parse {
                        message: "Expected 'recv' in select arm".into(),
                        line: recv.line,
                        column: recv.column,
                    });
                }
                self.expect(TokenType::Lparen, "Expected '(' after recv")?;
                let channel = self.parse_expr()?;
                self.expect(TokenType::Rparen, "Expected ')' after channel")?;
                self.expect(TokenType::FatArrow, "Expected '=>' in select arm")?;
                let body = if self.check(TokenType::Lbrace) {
                    self.advance();
                    let stmts = self.parse_block()?;
                    self.expect(TokenType::Rbrace, "Expected '}' to close select arm")?;
                    stmts
                } else {
                    vec![self.parse_stmt()?]
                };
                arms.push(crate::foundations::SelectArm {
                    channel,
                    body,
                    span: self.span_from(&arm_start, self.previous()),
                });
            }
            let end = self.expect(TokenType::Rbrace, "Expected '}' to close select")?;
            self.expect(TokenType::Semicolon, "Expected ';' after select")?;
            return Ok(Stmt::SelectStmt {
                arms,
                span: self.span_from(&start, &end),
            });
        }
        let expr = self.parse_expr()?;
        self.expect(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::ExprStmt {
            expr,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_subscribe_target(&mut self) -> Result<String, SpandaError> {
        let first = self.parse_label("Expected subscribe target")?;
        if self.match_types(&[TokenType::Dot]) {
            let second = self.parse_label("Expected member after '.'")?;
            Ok(format!("{first}.{second}"))
        } else {
            Ok(first)
        }
    }

    fn parse_discover_target(&mut self) -> Result<crate::comm::DiscoverTarget, SpandaError> {
        use crate::comm::DiscoverTarget;
        let name = self
            .expect(TokenType::Ident, "Expected discover target")?
            .lexeme;
        match name.as_str() {
            "robots" => Ok(DiscoverTarget::Robots),
            "agents" => Ok(DiscoverTarget::Agents),
            "devices" => Ok(DiscoverTarget::Devices),
            other => Err(SpandaError::Parse {
                message: format!("Expected robots, agents, or devices in discover, got '{other}'"),
                line: self.previous().line,
                column: self.previous().column,
            }),
        }
    }

    fn parse_discover_filter(
        &mut self,
    ) -> Result<Option<crate::comm::DiscoverFilter>, SpandaError> {
        if !self.match_types(&[TokenType::Where]) {
            return Ok(None);
        }
        self.expect(TokenType::Ident, "Expected 'capability' in discover filter")?;
        self.expect(
            TokenType::Includes,
            "Expected 'includes' in discover filter",
        )?;
        let cap = self
            .expect(TokenType::Ident, "Expected capability name")?
            .lexeme;
        Ok(Some(crate::comm::DiscoverFilter {
            capability: Some(cap),
        }))
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
        if self.check(TokenType::Ident)
            && self.peek().lexeme == "m"
            && self.tokens.get(self.pos + 1).map(|t| t.token_type) == Some(TokenType::Slash)
            && self.tokens.get(self.pos + 2).map(|t| t.lexeme.as_str()) == Some("s")
        {
            self.pos += 3;
            return Some(UnitKind::MPerS);
        }
        if self.check(TokenType::Ident)
            && self.peek().lexeme == "rad"
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
        if self.match_types(&[TokenType::Await]) {
            let start = self.previous().clone();
            let operand = self.parse_unary()?;
            return Ok(Expr::AwaitExpr {
                operand: Box::new(operand),
                span: self.span_from(&start, self.previous()),
            });
        }
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
                                self.expect(
                                    TokenType::Colon,
                                    "Expected ':' after struct field name",
                                )?;
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
                        let end =
                            self.expect(TokenType::Rbrace, "Expected '}' to close struct literal")?;
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
                let variant = self.parse_import_segment("Expected match arm variant")?;
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
        if self.match_types(&[TokenType::Call]) {
            let service = self.expect(TokenType::Ident, "Expected service name after call")?;
            self.expect(TokenType::Lparen, "Expected '(' after service name")?;
            self.expect(TokenType::Rparen, "Expected ')' after service arguments")?;
            return Ok(Expr::ServiceCallExpr {
                service_name: service.lexeme,
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Execute]) {
            let action = self.expect(TokenType::Ident, "Expected action name after execute")?;
            let goal = if self.match_types(&[TokenType::Lparen]) {
                let g = self.parse_expr()?;
                self.expect(TokenType::Rparen, "Expected ')' after execute goal")?;
                g
            } else {
                self.parse_expr()?
            };
            return Ok(Expr::ExecuteExpr {
                action_name: action.lexeme,
                goal: Box::new(goal),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Discover]) {
            let target = self.parse_discover_target()?;
            let filter = self.parse_discover_filter()?;
            return Ok(Expr::DiscoverExpr {
                target,
                filter,
                span: self.span_from(&start, self.previous()),
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
        if self.match_types(&[TokenType::Actuator]) {
            return Ok(Expr::IdentExpr {
                name: "actuator".into(),
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
            TokenType::Mission,
            TokenType::Duration,
            TokenType::Network,
            TokenType::Bandwidth,
            TokenType::Latency,
            TokenType::Timing,
            TokenType::Budget,
            TokenType::Fault,
            TokenType::Execute,
            TokenType::Discover,
            TokenType::Subscribe,
            TokenType::Receive,
            TokenType::Message,
            TokenType::Response,
            TokenType::Feedback,
            TokenType::Result,
            TokenType::Request,
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
            expr = re_span_expr(
                expr,
                Span {
                    start: loc(&start),
                    end: loc(&end),
                },
            );
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
        let lexeme = self.parse_binding_ident(message)?;
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
            && (self.check(TokenType::Ident)
                || self.check(TokenType::From)
                || self.check(TokenType::To)
                || self.check(TokenType::Goal))
    }

    fn parse_named_arg_name(&mut self) -> Result<String, SpandaError> {
        if self.match_types(&[TokenType::From]) {
            Ok("from".into())
        } else if self.match_types(&[TokenType::To]) {
            Ok("to".into())
        } else if self.match_types(&[TokenType::Goal]) {
            Ok("goal".into())
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
        | Expr::StructLiteralExpr { span, .. }
        | Expr::ServiceCallExpr { span, .. }
        | Expr::ExecuteExpr { span, .. }
        | Expr::DiscoverExpr { span, .. } => *span,
        Expr::AwaitExpr { span, .. } => *span,
    }
}

fn re_span_expr(expr: Expr, span: Span) -> Expr {
    match expr {
        Expr::LiteralExpr { value, .. } => Expr::LiteralExpr { value, span },
        Expr::UnitLiteralExpr { value, unit, .. } => Expr::UnitLiteralExpr { value, unit, span },
        Expr::IdentExpr { name, .. } => Expr::IdentExpr { name, span },
        Expr::BinaryExpr {
            op, left, right, ..
        } => Expr::BinaryExpr {
            op,
            left,
            right,
            span,
        },
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
        Expr::MemberExpr {
            object, property, ..
        } => Expr::MemberExpr {
            object,
            property,
            span,
        },
        Expr::MatchExpr {
            scrutinee, arms, ..
        } => Expr::MatchExpr {
            scrutinee,
            arms,
            span,
        },
        Expr::StructLiteralExpr {
            type_name, fields, ..
        } => Expr::StructLiteralExpr {
            type_name,
            fields,
            span,
        },
        Expr::ServiceCallExpr { service_name, .. } => Expr::ServiceCallExpr { service_name, span },
        Expr::ExecuteExpr {
            action_name, goal, ..
        } => Expr::ExecuteExpr {
            action_name,
            goal,
            span,
        },
        Expr::DiscoverExpr { target, filter, .. } => Expr::DiscoverExpr {
            target,
            filter: filter.clone(),
            span,
        },
        Expr::AwaitExpr { operand, span } => Expr::AwaitExpr {
            operand: Box::new((*operand).clone()),
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
        assert_eq!(
            binding,
            &Some(SensorBinding::Topic {
                path: "/scan".into()
            })
        );
        assert_eq!(actuators.len(), 1);
        assert!(safety.is_some());
        assert_eq!(behaviors.len(), 1);
    }

    #[test]
    fn parses_loop_every() {
        let ast =
            parse(tokenize("robot R { behavior b() { loop every 50ms { } } }").unwrap()).unwrap();
        let RobotDecl::RobotDecl { behaviors, .. } = &ast.robots()[0];
        let BehaviorDecl::BehaviorDecl { body, .. } = &behaviors[0];
        match &body[0] {
            Stmt::LoopStmt { interval_ms, .. } => assert_eq!(*interval_ms, 50.0),
            _ => panic!("expected loop"),
        }
    }

    #[test]
    fn parses_if_else() {
        let ast =
            parse(tokenize("robot R { behavior b() { if true { } else { } } }").unwrap()).unwrap();
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
