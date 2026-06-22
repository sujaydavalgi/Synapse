//! Spanda language parser — AST construction from token streams.
//!
use spanda_ast::foundations::*;
use spanda_ast::nodes::*;
use spanda_error::SpandaError;
use spanda_lexer::{unit_from_lexeme, Token, TokenType, TokenValue, UnitLexeme};
use spanda_regex_lang::RegexPattern;

pub fn parse(tokens: Vec<Token>) -> Result<Program, SpandaError> {
    // Parse input.
    //
    // Parameters:
    // - `tokens` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::parser::parse(tokens);

    // Produce parse program as the result.
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

type ContractClauses = (Option<Expr>, Option<Expr>, Option<Expr>);

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        // Create a new instance.
        //
        // Parameters:
        // - `tokens` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::parser::new(tokens);

        // Assemble the struct fields and return it.
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        // Peek.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &Token.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.peek();

        // Return pos] from this handle.
        &self.tokens[self.pos]
    }

    fn previous(&self) -> &Token {
        // Previous.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &Token.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.previous();

        // Return pos - 1] from this handle.
        &self.tokens[self.pos - 1]
    }

    fn advance(&mut self) -> Token {
        // Advance.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Token.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.advance();

        // take the branch when token type differs from Eof.
        if self.peek().token_type != TokenType::Eof {
            self.pos += 1;
        }
        self.tokens[self.pos - 1].clone()
    }

    fn check(&self, ty: TokenType) -> bool {
        // Check input.
        //
        // Parameters:
        // - `self` — method receiver
        // - `ty` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check(ty);

        // Call peek on the current instance.
        self.peek().token_type == ty
    }

    fn match_types(&mut self, types: &[TokenType]) -> bool {
        // Match types.
        //
        // Parameters:
        // - `self` — method receiver
        // - `types` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.match_types(types);

        // Process each type.
        for t in types {
            // Take this path when self.check(*t).
            if self.check(*t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expect(&mut self, ty: TokenType, message: &str) -> Result<Token, SpandaError> {
        // Expect.
        //
        // Parameters:
        // - `self` — method receiver
        // - `ty` — input value
        // - `message` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.expect(ty, message);

        // take this path when self.check(ty).
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
        // Span from.
        //
        // Parameters:
        // - `self` — method receiver
        // - `start` — input value
        // - `end` — input value
        //
        // Returns:
        // Span.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.span_from(start, end);

        // Produce Span as the result.
        Span {
            start: loc(start),
            end: loc(end),
        }
    }

    fn parse_binding_ident(&mut self, message: &str) -> Result<String, SpandaError> {
        // Parse binding ident.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_binding_ident(message);

        // Compute t for the following logic.
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

        // Take this path when ok.
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
        // Parse label.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_label(message);

        // Compute t for the following logic.
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

        // Take this path when ok.
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
        // Parse hal binding name.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_hal_binding_name(message);

        // Compute t for the following logic.
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

        // Take this path when ok.
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
        // Parse type name part.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_type_name_part(message);

        // take this path when self.check(TokenType::Ident).
        if self.check(TokenType::Ident) {
            return Ok(self.advance().lexeme);
        }
        self.parse_label(message)
    }

    fn parse_type_name(&mut self) -> Result<String, SpandaError> {
        // Parse type name.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_type_name();

        // Create mutable name for accumulating results.
        let mut name = self.parse_type_name_part("Expected type name")?;

        // Take this path when self.check(TokenType::Lt).
        if self.check(TokenType::Lt) {
            name = self.finish_generic_type_name(name)?;
        }
        Ok(name)
    }

    fn finish_generic_type_name(&mut self, base: String) -> Result<String, SpandaError> {
        // Finish generic type name.
        //
        // Parameters:
        // - `self` — method receiver
        // - `base` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.finish_generic_type_name(base);

        // Call expect on the current instance.
        self.expect(TokenType::Lt, "Expected '<' to open generic type")?;
        let mut args = Vec::new();

        // Take the branch when Gt) is false.
        if !self.check(TokenType::Gt) {
            // Run the loop body until it exits.
            loop {
                args.push(self.parse_type_name()?);

                // Take the branch when Comma]) is false.
                if !self.match_types(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.expect(TokenType::Gt, "Expected '>' to close generic type")?;
        Ok(format!("{base}<{args}>", args = args.join(", ")))
    }

    fn parse_type_annotation(&mut self) -> Result<SpandaType, SpandaError> {
        // Parse type annotation.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_type_annotation();

        // Import the items needed by the logic below.
        use spanda_typecheck::type_system::{resolve_generic_type, resolve_type_name};
        let start = self.peek().clone();

        // Take this path when self.match types(&[TokenType::Dyn]).
        if self.match_types(&[TokenType::Dyn]) {
            let trait_name = self.parse_type_name_part("Expected trait name after dyn")?;
            return Ok(SpandaType::TraitObject { trait_name });
        }
        let mut parts = vec![self.parse_type_name_part("Expected type name")?];

        // Repeat while self.match types(&[TokenType::Dot]).
        while self.match_types(&[TokenType::Dot]) {
            parts.push(self.parse_type_name_part("Expected type name after '.'")?);
        }
        let qualified = parts.join(".");

        // Take this path when self.match types(&[TokenType::Lt]).
        if self.match_types(&[TokenType::Lt]) {
            let mut args = Vec::new();

            // Take the branch when Gt) is false.
            if !self.check(TokenType::Gt) {
                // Run the loop body until it exits.
                loop {
                    args.push(self.parse_type_annotation()?);

                    // Take the branch when Comma]) is false.
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
            // Match on resolve type name and handle each case.
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
        // Parse program.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_program();

        // Compute start for the following logic.
        let start = self.peek().clone();
        let mut module_name = None;

        // Take this path when self.check(TokenType::Module).
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
        let mut extern_functions = Vec::new();
        let mut tests = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut hardware_profiles = Vec::new();
        let mut deployments = Vec::new();
        let mut requires_hardware = None;
        let mut requires_network = None;
        let mut requires_connectivity = None;
        let mut geofences = Vec::new();
        let mut fleets = Vec::new();
        let mut swarms = Vec::new();
        let mut program_safety_zones = Vec::new();
        let mut certifications = Vec::new();
        let mut connectivity_policies = Vec::new();
        let mut ble_services = Vec::new();
        let mut simulate_compatibility = None;
        let mut messages = Vec::new();
        let mut validate_rules = Vec::new();
        let mut robots = Vec::new();

        // Repeat while self.check(TokenType::Import).
        while self.check(TokenType::Import) {
            imports.push(self.parse_import()?);
        }

        // Repeat while !self.check(TokenType::Eof).
        while !self.check(TokenType::Eof) {
            // Take this path when self.is module fn start().
            if self.is_module_fn_start() {
                functions.push(self.parse_module_fn()?);
            } else if self.match_types(&[TokenType::Extern]) {
                extern_functions.push(self.parse_extern_fn()?);
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
            } else if self.check(TokenType::RequiresConnectivity) {
                requires_connectivity = Some(self.parse_requires_connectivity()?);
            } else if self.check(TokenType::Geofence) {
                geofences.push(self.parse_geofence()?);
            } else if self.check(TokenType::Fleet) {
                fleets.push(self.parse_fleet()?);
            } else if self.check(TokenType::Swarm) {
                swarms.push(self.parse_swarm()?);
            } else if self.check(TokenType::SafetyZone) {
                program_safety_zones.push(self.parse_program_safety_zone()?);
            } else if self.check(TokenType::Certify) {
                certifications.push(self.parse_certify()?);
            } else if self.check(TokenType::ConnectivityPolicy) {
                connectivity_policies.push(self.parse_connectivity_policy()?);
            } else if self.check(TokenType::BleService) {
                ble_services.push(self.parse_ble_service()?);
            } else if self.check(TokenType::SimulateCompatibility) {
                simulate_compatibility = Some(self.parse_simulate_compatibility()?);
            } else if self.check(TokenType::Message) {
                messages.push(self.parse_message()?);
            } else if self.check(TokenType::Ident) && self.peek().lexeme == "validate" {
                validate_rules.push(self.parse_validate_rule()?);
            } else if self.check(TokenType::Robot) {
                robots.push(self.parse_robot()?);
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected struct, enum, trait, hardware, deploy, validate, or robot declaration"
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
            extern_functions,
            structs,
            enums,
            traits,
            hardware_profiles,
            deployments,
            requires_hardware,
            requires_network,
            requires_connectivity,
            geofences,
            fleets,
            swarms,
            program_safety_zones,
            certifications,
            connectivity_policies,
            ble_services,
            simulate_compatibility,
            messages,
            validate_rules,
            robots,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_hardware(&mut self) -> Result<HardwareDecl, SpandaError> {
        // Parse hardware.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_hardware();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::HardwareDecl;
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
        let mut connectivity = Vec::new();
        let mut battery_wh = None;
        let mut network_bandwidth_mbps = None;
        let mut network_latency_ms = None;
        let mut min_control_period_ms = None;
        let mut power_draw_w = None;

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Cpu]).
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

                // Take this path when self.check(TokenType::True).
                if self.check(TokenType::True) {
                    self.advance();
                    gpu_required = true;
                } else {
                    gpu_tops = Some(self.parse_number_value()?);

                    // Take the branch when lexeme equals "TOPS".
                    if self.check(TokenType::Ident) && self.peek().lexeme == "TOPS" {
                        self.advance();
                    }
                }
                self.expect(TokenType::Semicolon, "Expected ';' after gpu")?;
            } else if self.match_types(&[TokenType::Sensors]) {
                sensors = self.parse_hardware_type_list("sensors")?;
            } else if self.match_types(&[TokenType::Actuators]) {
                actuators = self.parse_hardware_type_list("actuators")?;
            } else if self.match_types(&[TokenType::Connectivity]) {
                connectivity = self.parse_hardware_type_list("connectivity")?;
            } else if self.match_types(&[TokenType::Battery]) {
                self.expect(TokenType::Lbrace, "Expected '{' after battery")?;

                // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
                while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                    // Take this path when self.match types(&[TokenType::Capacity]).
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

                // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
                while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                    // Take this path when self.match types(&[TokenType::Bandwidth]).
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

                // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
                while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                    // Take this path when self.match types(&[TokenType::MinPeriod]).
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

                // Take the branch when lexeme equals "W".
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
            connectivity,
            battery_wh,
            network_bandwidth_mbps,
            network_latency_ms,
            min_control_period_ms,
            power_draw_w,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_hardware_type_list(&mut self, kind: &str) -> Result<Vec<String>, SpandaError> {
        // Parse hardware type list.
        //
        // Parameters:
        // - `self` — method receiver
        // - `kind` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_hardware_type_list(kind);

        // Call expect on the current instance.
        self.expect(TokenType::Lbracket, &format!("Expected '[' after {kind}"))?;
        let mut items = Vec::new();

        // Take the branch when Rbracket) is false.
        if !self.check(TokenType::Rbracket) {
            // Run the loop body until it exits.
            loop {
                items.push(self.parse_label(&format!("Expected {kind} type name"))?);

                // Take this path when self.match types(&[TokenType::Comma]).
                if self.match_types(&[TokenType::Comma]) {
                    // Take this path when self.check(TokenType::Rbracket).
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
        // Parse storage amount.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_storage_amount();

        // Compute the value consumed by the next step.
        let value = self.parse_number_value()?;

        // Take this path when self.check(TokenType::Ident).
        if self.check(TokenType::Ident) {
            let unit = self.peek().lexeme.as_str();
            let mb = match unit {
                "GB" | "Gb" => value * 1024.0,
                "MB" | "Mb" => value,
                "TB" | "Tb" => value * 1024.0 * 1024.0,
                _ => value,
            };

            // Take the branch when unit equals "GB".
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

    fn parse_signed_number_value(&mut self) -> Result<f64, SpandaError> {
        let mut sign = 1.0;
        if self.match_types(&[TokenType::Minus]) {
            sign = -1.0;
        }
        Ok(sign * self.parse_number_value()?)
    }

    fn parse_connectivity_link(&mut self, message: &str) -> Result<String, SpandaError> {
        let t = self.peek();
        let name = match t.token_type {
            TokenType::Ident => self.advance().lexeme.clone(),
            TokenType::Bluetooth => {
                self.advance();
                "bluetooth".into()
            }
            TokenType::Network => {
                self.advance();
                "network".into()
            }
            _ => {
                return Err(SpandaError::Parse {
                    message: message.into(),
                    line: t.line,
                    column: t.column,
                });
            }
        };
        Ok(name)
    }

    fn parse_trigger_domain(&mut self) -> Result<String, SpandaError> {
        self.parse_connectivity_link("Expected trigger domain name")
    }

    fn parse_number_value(&mut self) -> Result<f64, SpandaError> {
        // Parse number value.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_number_value();

        // Compute tok for the following logic.
        let tok = self.expect(TokenType::Number, "Expected number")?;

        // Match on value and handle each case.
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
        // Parse deploy.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_deploy();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::DeployDecl;
        let start = self.advance();
        let robot_name = self.parse_label("Expected robot name after deploy")?;
        self.expect(TokenType::To, "Expected 'to' after deploy robot name")?;
        let mut targets = Vec::new();

        // Take this path when self.match types(&[TokenType::Lbracket]).
        if self.match_types(&[TokenType::Lbracket]) {
            // Take the branch when Rbracket) is false.
            if !self.check(TokenType::Rbracket) {
                // Run the loop body until it exits.
                loop {
                    targets.push(self.parse_label("Expected hardware target name")?);

                    // Take the branch when Comma]) is false.
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
        // Parse network amount.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_network_amount();

        // Compute the value consumed by the next step.
        let value = self.parse_number_value()?;

        // Take this path when self.check(TokenType::Ident).
        if self.check(TokenType::Ident) {
            let unit = self.peek().lexeme.as_str();

            // Take the branch when unit equals "Mbps" || unit == "mbps".
            if unit == "Mbps" || unit == "mbps" {
                self.advance();
                return Ok(value);
            }

            // Take the branch when unit equals "Gbps" || unit == "gbps".
            if unit == "Gbps" || unit == "gbps" {
                self.advance();
                return Ok(value * 1000.0);
            }
        }
        Ok(value)
    }

    fn parse_requires_hardware(
        &mut self,
    ) -> Result<spanda_ast::foundations::RequiresHardwareDecl, SpandaError> {
        // Parse requires hardware.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_requires_hardware();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::RequiresHardwareDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after requires_hardware")?;
        let mut memory_mb_min = None;
        let mut storage_mb_min = None;
        let mut gpu_tops_min = None;
        let mut gpu_required = false;
        let mut sensors = Vec::new();
        let mut actuators = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Memory]).
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
                // Take this path when self.check(TokenType::Gte).
                if self.check(TokenType::Gte) {
                    self.advance();
                    gpu_tops_min = Some(self.parse_number_value()?);

                    // Take the branch when lexeme equals "TOPS".
                    if self.check(TokenType::Ident) && self.peek().lexeme == "TOPS" {
                        self.advance();
                    }
                } else {
                    self.expect(TokenType::Colon, "Expected ':' or '>=' after gpu")?;

                    // Take this path when self.match types(&[TokenType::True]).
                    if self.match_types(&[TokenType::True]) {
                        gpu_required = true;
                    } else {
                        gpu_tops_min = Some(self.parse_number_value()?);

                        // Take the branch when lexeme equals "TOPS".
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
    ) -> Result<spanda_ast::foundations::RequiresNetworkDecl, SpandaError> {
        // Parse requires network.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_requires_network();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::RequiresNetworkDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after requires_network")?;
        let mut bandwidth_mbps_min = None;
        let mut latency_ms_max = None;

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Bandwidth]).
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

    fn parse_requires_connectivity(
        &mut self,
    ) -> Result<spanda_ast::foundations::RequiresConnectivityDecl, SpandaError> {
        use spanda_ast::foundations::RequiresConnectivityDecl;
        use spanda_connectivity_runtime::ConnectivityRequirement;
        let start = self.advance();
        self.expect(
            TokenType::Lbrace,
            "Expected '{' after requires_connectivity",
        )?;
        let mut channels = Vec::new();
        let mut latency_ms_max = None;
        let mut bandwidth_mbps_min = None;
        let mut packet_loss_pct_max = None;

        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Latency]) {
                self.expect(TokenType::Lte, "Expected '<=' after latency")?;
                latency_ms_max = Some(self.parse_duration()?);
                self.expect(TokenType::Semicolon, "Expected ';' after latency")?;
            } else if self.match_types(&[TokenType::Bandwidth]) {
                self.expect(TokenType::Gte, "Expected '>=' after bandwidth")?;
                bandwidth_mbps_min = Some(self.parse_network_amount()?);
                self.expect(TokenType::Semicolon, "Expected ';' after bandwidth")?;
            } else if self.match_types(&[TokenType::PacketLoss]) {
                self.expect(TokenType::Lte, "Expected '<=' after packet_loss")?;
                packet_loss_pct_max = Some(self.parse_number_value()?);
                if self.check(TokenType::Percent) {
                    self.advance();
                }
                self.expect(TokenType::Semicolon, "Expected ';' after packet_loss")?;
            } else if self.check(TokenType::Ident) {
                let key = self.advance().lexeme.clone();
                self.expect(TokenType::Colon, "Expected ':' after connectivity key")?;
                let level = if self.check(TokenType::Ident) && self.peek().lexeme == "required" {
                    self.advance();
                    ConnectivityRequirement::Required
                } else if self.check(TokenType::Ident) && self.peek().lexeme == "optional" {
                    self.advance();
                    ConnectivityRequirement::Optional
                } else {
                    let t = self.peek();
                    return Err(SpandaError::Parse {
                        message: "Expected required or optional after connectivity key".into(),
                        line: t.line,
                        column: t.column,
                    });
                };
                self.expect(
                    TokenType::Semicolon,
                    "Expected ';' after connectivity requirement",
                )?;
                channels.push((key, level));
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected connectivity channel or metric in requires_connectivity"
                        .into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(
            TokenType::Rbrace,
            "Expected '}' to close requires_connectivity",
        )?;
        Ok(RequiresConnectivityDecl::RequiresConnectivityDecl {
            channels,
            latency_ms_max,
            bandwidth_mbps_min,
            packet_loss_pct_max,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_geofence(&mut self) -> Result<spanda_ast::foundations::GeofenceDecl, SpandaError> {
        use spanda_ast::foundations::GeofenceDecl;
        let start = self.advance();
        let name = self
            .expect(TokenType::Ident, "Expected geofence name")?
            .lexeme;
        self.expect(TokenType::Lbrace, "Expected '{' after geofence name")?;
        let mut center_lat = None;
        let mut center_lon = None;
        let mut radius_m = None;

        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.check(TokenType::Ident) && self.peek().lexeme == "center" {
                self.advance();
                self.expect(TokenType::Colon, "Expected ':' after center")?;
                self.expect(TokenType::Ident, "Expected geo")?;
                self.expect(TokenType::Lparen, "Expected '(' after geo")?;
                center_lat = Some(self.parse_signed_number_value()?);
                self.expect(TokenType::Comma, "Expected ',' in geo()")?;
                center_lon = Some(self.parse_signed_number_value()?);
                self.expect(TokenType::Rparen, "Expected ')' after geo coordinates")?;
                self.expect(TokenType::Semicolon, "Expected ';' after center")?;
            } else if self.match_types(&[TokenType::Radius]) {
                self.expect(TokenType::Colon, "Expected ':' after radius")?;
                radius_m = Some(self.parse_number_value()?);
                if self.check(TokenType::Ident) && self.peek().lexeme == "m" {
                    self.advance();
                }
                self.expect(TokenType::Semicolon, "Expected ';' after radius")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected center or radius in geofence block".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close geofence")?;
        Ok(GeofenceDecl::GeofenceDecl {
            name,
            center_lat: center_lat.unwrap_or(0.0),
            center_lon: center_lon.unwrap_or(0.0),
            radius_m: radius_m.unwrap_or(0.0),
            span: self.span_from(&start, &end),
        })
    }

    fn parse_connectivity_policy(
        &mut self,
    ) -> Result<spanda_ast::foundations::ConnectivityPolicyDecl, SpandaError> {
        use spanda_ast::foundations::ConnectivityPolicyDecl;
        let start = self.advance();
        let name = self
            .expect(TokenType::Ident, "Expected connectivity policy name")?
            .lexeme;
        self.expect(TokenType::Lbrace, "Expected '{' after policy name")?;
        let mut preferred = None;
        let mut fallback = None;
        let mut emergency = None;
        let mut switch_if_latency_ms = None;
        let mut switch_if_packet_loss_pct = None;

        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.check(TokenType::Ident) && self.peek().lexeme == "preferred" {
                self.advance();
                self.expect(TokenType::Colon, "Expected ':' after preferred")?;
                preferred = Some(self.parse_connectivity_link("Expected link name")?);
                self.expect(TokenType::Semicolon, "Expected ';' after preferred")?;
            } else if self.match_types(&[TokenType::Fallback]) {
                self.expect(TokenType::Colon, "Expected ':' after fallback")?;
                fallback = Some(self.parse_connectivity_link("Expected link name")?);
                self.expect(TokenType::Semicolon, "Expected ';' after fallback")?;
            } else if self.check(TokenType::Ident) && self.peek().lexeme == "emergency" {
                self.advance();
                self.expect(TokenType::Colon, "Expected ':' after emergency")?;
                emergency = Some(self.parse_connectivity_link("Expected link name")?);
                self.expect(TokenType::Semicolon, "Expected ';' after emergency")?;
            } else if self.match_types(&[TokenType::SwitchIf]) {
                if self.match_types(&[TokenType::Latency]) {
                    self.expect(TokenType::Gt, "Expected '>' after latency")?;
                    switch_if_latency_ms = Some(self.parse_duration()?);
                } else if self.match_types(&[TokenType::PacketLoss]) {
                    self.expect(TokenType::Gt, "Expected '>' after packet_loss")?;
                    switch_if_packet_loss_pct = Some(self.parse_number_value()?);
                    if self.check(TokenType::Percent) {
                        self.advance();
                    }
                } else {
                    let t = self.peek();
                    return Err(SpandaError::Parse {
                        message: "Expected latency or packet_loss after switch_if".into(),
                        line: t.line,
                        column: t.column,
                    });
                }
                self.expect(TokenType::Semicolon, "Expected ';' after switch_if")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected policy member in connectivity_policy".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(
            TokenType::Rbrace,
            "Expected '}' to close connectivity_policy",
        )?;
        Ok(ConnectivityPolicyDecl::ConnectivityPolicyDecl {
            name,
            preferred: preferred.unwrap_or_default(),
            fallback: fallback.unwrap_or_default(),
            emergency,
            switch_if_latency_ms,
            switch_if_packet_loss_pct,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_ble_service(
        &mut self,
    ) -> Result<spanda_ast::foundations::BleServiceDecl, SpandaError> {
        use spanda_ast::foundations::BleServiceDecl;
        let start = self.advance();
        let name = self
            .expect(TokenType::Ident, "Expected BLE service name")?
            .lexeme;
        self.expect(TokenType::Lbrace, "Expected '{' after ble_service name")?;
        let mut uuid = None;
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.check(TokenType::Ident) && self.peek().lexeme == "uuid" {
                self.advance();
                self.expect(TokenType::Colon, "Expected ':' after uuid")?;
                let u = self.expect(TokenType::String, "Expected UUID string")?;
                uuid = Some(u.lexeme.trim_matches('"').to_string());
                self.expect(TokenType::Semicolon, "Expected ';' after uuid")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected uuid in ble_service block".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close ble_service")?;
        Ok(BleServiceDecl::BleServiceDecl {
            name,
            uuid: uuid.unwrap_or_default(),
            span: self.span_from(&start, &end),
        })
    }

    fn parse_bluetooth_config(
        &mut self,
    ) -> Result<spanda_ast::foundations::BluetoothConfigDecl, SpandaError> {
        use spanda_ast::foundations::BluetoothConfigDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after bluetooth")?;
        let mut scan_pattern = None;
        let mut pair_mode = None;
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.check(TokenType::Ident) && self.peek().lexeme == "scan" {
                self.advance();
                self.expect(TokenType::For, "Expected 'for' after scan")?;
                self.expect(TokenType::Ident, "Expected 'devices'")?;
                self.expect(TokenType::Where, "Expected 'where' in bluetooth scan")?;
                self.expect(TokenType::Ident, "Expected 'name'")?;
                self.expect(TokenType::Matches, "Expected 'matches' in bluetooth scan")?;
                scan_pattern = Some(self.parse_regex_literal()?);
                self.expect(TokenType::Semicolon, "Expected ';' after scan")?;
            } else if self.check(TokenType::Ident) && self.peek().lexeme == "pair" {
                self.advance();
                pair_mode = Some(if self.match_types(&[TokenType::TrustedOnly]) {
                    "trusted_only".into()
                } else {
                    self.parse_label("Expected pair mode")?
                });
                self.expect(TokenType::Semicolon, "Expected ';' after pair")?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected scan or pair in bluetooth block".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close bluetooth")?;
        Ok(BluetoothConfigDecl::BluetoothConfigDecl {
            scan_pattern,
            pair_mode,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_simulate_compatibility(
        &mut self,
    ) -> Result<spanda_ast::foundations::SimulateCompatibilityDecl, SpandaError> {
        // Parse simulate compatibility.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_simulate_compatibility();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::{SimFaultDecl, SimulateCompatibilityDecl};
        let start = self.advance();
        self.expect(
            TokenType::Lbrace,
            "Expected '{' after simulate_compatibility",
        )?;
        let mut faults = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Fault]).
            if self.match_types(&[TokenType::Fault]) {
                let fault_start = self.peek().clone();
                let fault_type = self.parse_label("Expected fault type name")?;
                let mut at_offset_ms = None;
                let mut duration_ms = None;
                if self.check(TokenType::At)
                    || (self.check(TokenType::Ident) && self.peek().lexeme == "at")
                {
                    self.advance();
                    if self.check(TokenType::Ident) && self.peek().lexeme.starts_with("T+") {
                        let offset = self.advance().lexeme.clone();
                        if let Some(secs_str) =
                            offset.strip_prefix("T+").and_then(|s| s.strip_suffix('s'))
                        {
                            at_offset_ms = secs_str.parse::<f64>().ok().map(|s| s * 1000.0);
                        }
                    } else if self.check(TokenType::Ident) && self.peek().lexeme == "T" {
                        self.advance();
                        if self.match_types(&[TokenType::Plus]) {
                            if self.check(TokenType::UnitLiteral) {
                                let offset = self.advance().lexeme.clone();
                                if let Some(secs_str) = offset.strip_suffix('s') {
                                    at_offset_ms = secs_str.parse::<f64>().ok().map(|s| s * 1000.0);
                                }
                            } else if self.check(TokenType::Number) {
                                let secs = self.advance().lexeme.parse::<f64>().ok();
                                at_offset_ms = secs.map(|s| s * 1000.0);
                            } else {
                                let num = self.parse_label("Expected fault offset")?;
                                if let Some(secs_str) = num.strip_suffix('s') {
                                    at_offset_ms = secs_str.parse::<f64>().ok().map(|s| s * 1000.0);
                                }
                            }
                        }
                    }
                } else if self.match_types(&[TokenType::Duration]) {
                    duration_ms = Some(self.parse_duration()?);
                }
                self.expect(TokenType::Semicolon, "Expected ';' after fault")?;
                faults.push(SimFaultDecl {
                    fault_type,
                    at_offset_ms,
                    duration_ms,
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

    fn parse_mission(&mut self) -> Result<spanda_ast::foundations::MissionDecl, SpandaError> {
        // Parse mission.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_mission();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::MissionDecl;
        let start = self.advance();
        let name = if self.check(TokenType::Ident) {
            Some(self.advance().lexeme)
        } else {
            None
        };
        self.expect(TokenType::Lbrace, "Expected '{' after mission")?;
        let mut duration_hours = None;
        let mut steps = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Duration]).
            if self.match_types(&[TokenType::Duration]) {
                self.expect(TokenType::Colon, "Expected ':' after duration")?;
                duration_hours = Some(self.parse_duration_hours()?);
                self.expect(TokenType::Semicolon, "Expected ';' after duration")?;
            } else {
                let step = self.parse_label("Expected mission step name")?;
                self.expect(TokenType::Semicolon, "Expected ';' after mission step")?;
                steps.push(step);
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close mission")?;
        if duration_hours.is_none() && steps.is_empty() {
            let t = self.peek();
            return Err(SpandaError::Parse {
                message: "mission block requires duration or at least one step".into(),
                line: t.line,
                column: t.column,
            });
        }
        Ok(MissionDecl::MissionDecl {
            name,
            duration_hours,
            steps,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_fleet(&mut self) -> Result<spanda_ast::robotics_decl::FleetDecl, SpandaError> {
        // Parse fleet.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_fleet();

        use spanda_ast::robotics_decl::FleetDecl;
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected fleet name")?.lexeme;
        self.expect(TokenType::Lbrace, "Expected '{' after fleet name")?;
        let mut members = Vec::new();

        // Collect robot member identifiers until the closing brace.
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let member = self
                .expect(TokenType::Ident, "Expected robot name in fleet block")?
                .lexeme;
            self.expect(TokenType::Semicolon, "Expected ';' after fleet member")?;
            members.push(member);
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close fleet")?;
        Ok(FleetDecl::FleetDecl {
            name,
            members,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_swarm(&mut self) -> Result<spanda_ast::robotics_decl::SwarmDecl, SpandaError> {
        // Parse a program-level swarm coordinator declaration.
        use spanda_ast::robotics_decl::{SwarmDecl, SwarmPolicy};
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected swarm name")?.lexeme;
        self.expect(TokenType::Lbrace, "Expected '{' after swarm name")?;
        let mut fleet_name = None;
        let mut policy = SwarmPolicy::RoundRobin;

        // Parse fleet and policy fields inside the swarm block.
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.check(TokenType::Fleet) {
                self.advance();
                fleet_name = Some(
                    self.expect(TokenType::Ident, "Expected fleet name after 'fleet'")?
                        .lexeme,
                );
                self.expect(TokenType::Semicolon, "Expected ';' after fleet name")?;
            } else if self.match_types(&[TokenType::Policy]) {
                let policy_name = self
                    .expect(TokenType::Ident, "Expected swarm policy name")?
                    .lexeme;
                self.expect(TokenType::Semicolon, "Expected ';' after swarm policy")?;
                policy = SwarmPolicy::parse_ident(&policy_name).ok_or_else(|| {
                    let t = self.previous();
                    SpandaError::Parse {
                        message: format!(
                            "Unknown swarm policy '{policy_name}' (expected round_robin, broadcast, or leader_follow)"
                        ),
                        line: t.line,
                        column: t.column,
                    }
                })?;
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected fleet or policy in swarm block".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close swarm")?;
        let fleet_name = fleet_name.ok_or_else(|| SpandaError::Parse {
            message: format!("swarm '{name}' requires a fleet reference"),
            line: end.line,
            column: end.column,
        })?;
        Ok(SwarmDecl::SwarmDecl {
            name,
            fleet_name,
            policy,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_program_safety_zone(
        &mut self,
    ) -> Result<spanda_ast::robotics_decl::ProgramSafetyZoneDecl, SpandaError> {
        // Parse program safety zone.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_program_safety_zone();

        use spanda_ast::robotics_decl::ProgramSafetyZoneDecl;
        let start = self.advance();
        let name = self
            .expect(TokenType::Ident, "Expected safety zone name")?
            .lexeme;
        self.expect(TokenType::Lbrace, "Expected '{' after safety zone name")?;
        let mut max_speed_mps = None;

        // Parse optional max_speed policy entries inside the zone block.
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            if self.match_types(&[TokenType::Ident]) && self.previous().lexeme == "max_speed" {
                let value = self.parse_expr()?;
                self.expect(TokenType::Semicolon, "Expected ';' after max_speed")?;
                max_speed_mps = Some(self.expr_to_mps(&value)?);
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected max_speed in safety_zone block".into(),
                    line: t.line,
                    column: t.column,
                });
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close safety_zone")?;
        Ok(ProgramSafetyZoneDecl::ProgramSafetyZoneDecl {
            name,
            max_speed_mps,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_certify(&mut self) -> Result<spanda_ast::robotics_decl::CertifyDecl, SpandaError> {
        // Parse program-level certification metadata.
        use spanda_ast::robotics_decl::{CertificationStandard, CertifyDecl};
        let start = self.advance();
        let standard_name = self
            .expect(
                TokenType::Ident,
                "Expected certification standard after 'certify'",
            )?
            .lexeme;
        let standard = CertificationStandard::parse_ident(&standard_name).ok_or_else(|| {
            SpandaError::Parse {
                message: format!(
                    "Unknown certification standard '{standard_name}' (expected ISO13849, IEC61508, or ISO26262)"
                ),
                line: start.line,
                column: start.column,
            }
        })?;
        let mut level = None;
        if self.check(TokenType::Lbrace) {
            self.advance();
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                if self.check(TokenType::Ident) && self.peek().lexeme == "level" {
                    self.advance();
                    level = Some(
                        self.expect(
                            TokenType::Ident,
                            "Expected certification level after 'level'",
                        )?
                        .lexeme,
                    );
                    self.expect(
                        TokenType::Semicolon,
                        "Expected ';' after certification level",
                    )?;
                } else {
                    let t = self.peek();
                    return Err(SpandaError::Parse {
                        message: "Expected 'level <Level>;' in certify block".into(),
                        line: t.line,
                        column: t.column,
                    });
                }
            }
            self.expect(TokenType::Rbrace, "Expected '}' to close certify block")?;
        } else {
            self.expect(
                TokenType::Semicolon,
                "Expected ';' after certification standard",
            )?;
        }
        let end = self.previous();
        Ok(CertifyDecl::CertifyDecl {
            standard,
            level,
            span: self.span_from(&start, end),
        })
    }

    fn expr_to_mps(&self, expr: &Expr) -> Result<f64, SpandaError> {
        // Convert a max_speed expression to meters per second.
        if let Expr::UnitLiteralExpr { value, unit, span } = expr {
            let mps = match unit {
                UnitKind::MPerS => *value,
                UnitKind::KmPerH => *value / 3.6,
                UnitKind::Mph => *value * 0.44704,
                _ => {
                    return Err(SpandaError::Parse {
                        message: "max_speed requires a velocity unit (m/s, km/h, mph)".into(),
                        line: span.start.line,
                        column: span.start.column,
                    });
                }
            };
            Ok(mps)
        } else {
            let span = match expr {
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
                | Expr::DiscoverExpr { span, .. }
                | Expr::AwaitExpr { span, .. }
                | Expr::SpawnExpr { span, .. } => span,
            };
            Err(SpandaError::Parse {
                message: "max_speed requires a numeric velocity literal".into(),
                line: span.start.line,
                column: span.start.column,
            })
        }
    }

    fn parse_duration_hours(&mut self) -> Result<f64, SpandaError> {
        // Parse duration hours.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_duration_hours();

        // take this path when self.check(TokenType::UnitLiteral).
        if self.check(TokenType::UnitLiteral) {
            let tok = self.advance();

            // Take this path when let (TokenValue::Number(n), Some(unit lex)) = (tok.value, tok.unit).
            if let (TokenValue::Number(n), Some(unit_lex)) = (tok.value, tok.unit) {
                return Ok(Self::duration_to_hours(n, unit_from_lexeme(unit_lex)));
            }
        }
        let value = self.parse_number_value()?;

        // Take this path when self.check(TokenType::Ident).
        if self.check(TokenType::Ident) {
            let unit = self.peek().lexeme.as_str();
            let hours = match unit {
                "h" | "hr" | "hrs" | "hour" | "hours" => value,
                "min" | "mins" | "minute" | "minutes" => value / 60.0,
                "s" | "sec" | "secs" => value / 3600.0,
                _ => value,
            };

            // Take the branch when unit equals "h".
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
        // Duration to hours.
        //
        // Parameters:
        // - `value` — input value
        // - `unit` — input value
        //
        // Returns:
        // Numeric result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::parser::duration_to_hours(value, unit);

        // Match on unit and handle each case.
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
        // Parse energy wh value.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_energy_wh_value();

        // take this path when self.check(TokenType::UnitLiteral).
        if self.check(TokenType::UnitLiteral) {
            let tok = self.advance();

            // Take this path when let (TokenValue::Number(n), Some(unit lex)) = (tok.value, tok.unit).
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

        // Take this path when self.check(TokenType::Ident).
        if self.check(TokenType::Ident) {
            let unit = self.peek().lexeme.as_str();
            let wh = match unit {
                "Wh" => value,
                "kWh" => value * 1000.0,
                "J" => value / 3600.0,
                _ => value,
            };

            // Take the branch when unit equals "Wh" || unit == "kWh" || unit == "J".
            if unit == "Wh" || unit == "kWh" || unit == "J" {
                self.advance();
            }
            return Ok(wh);
        }
        Ok(value)
    }

    fn parse_budget(&mut self) -> Result<spanda_ast::foundations::ResourceBudgetDecl, SpandaError> {
        // Parse budget.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_budget();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::ResourceBudgetDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after budget")?;
        let mut battery_pct_max = None;
        let mut memory_mb_max = None;
        let mut cpu_pct_max = None;
        let mut network_mbps_max = None;
        let mut storage_mb_max = None;
        let mut gpu_pct_max = None;

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Battery]).
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
            } else if self.match_types(&[TokenType::Gpu]) {
                self.expect(TokenType::Lte, "Expected '<=' after gpu in budget")?;
                gpu_pct_max = Some(self.parse_percent_value()?);
                self.expect(TokenType::Semicolon, "Expected ';' after gpu budget")?;
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
            gpu_pct_max,
            network_mbps_max,
            storage_mb_max,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_percent_value(&mut self) -> Result<f64, SpandaError> {
        // Parse percent value.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_percent_value();

        // Compute the value consumed by the next step.
        let value = self.parse_number_value()?;

        // Take the branch when lexeme equals "%".
        if self.check(TokenType::Ident) && self.peek().lexeme == "%" {
            self.advance();
        } else if self.match_types(&[TokenType::Percent]) {
        }
        Ok(value)
    }

    fn parse_dotted_name(&mut self, message: &str) -> Result<String, SpandaError> {
        // Parse dotted name.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_dotted_name(message);

        // Create mutable parts for accumulating results.
        let mut parts = vec![self.parse_import_segment(message)?];

        // Repeat while self.match types(&[TokenType::Dot]).
        while self.match_types(&[TokenType::Dot]) {
            parts.push(self.parse_import_segment("Expected name after '.'")?);
        }
        Ok(parts.join("."))
    }

    fn parse_import(&mut self) -> Result<ImportDecl, SpandaError> {
        // Parse import.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_import();

        // Compute start for the following logic.
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
        // Parse import segment.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_import_segment(message);

        // take this path when self.check(TokenType::Ident).
        if self.check(TokenType::Ident) {
            return Ok(self.advance().lexeme);
        }

        // Take this path when self.check(TokenType::Eof).
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
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_module_fn_start();

        // Call check on the current instance.
        self.check(TokenType::Export)
            || self.check(TokenType::Public)
            || self.check(TokenType::Private)
            || self.check(TokenType::Async)
            || self.check(TokenType::Fn)
    }

    fn parse_type_params(&mut self) -> Result<Vec<String>, SpandaError> {
        // Parse type params.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_type_params();

        // take the branch when Lt]) is false.
        if !self.match_types(&[TokenType::Lt]) {
            return Ok(Vec::new());
        }
        let mut params = Vec::new();

        // Run the loop body until it exits.
        loop {
            params.push(self.parse_label("Expected type parameter name")?);

            // Take the branch when Comma]) is false.
            if !self.match_types(&[TokenType::Comma]) {
                break;
            }
        }
        self.expect(TokenType::Gt, "Expected '>' after type parameters")?;
        Ok(params)
    }

    fn parse_module_fn(&mut self) -> Result<spanda_ast::foundations::ModuleFnDecl, SpandaError> {
        // Parse module fn.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_module_fn();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::{ModuleFnDecl, ModuleParamDecl, Visibility};
        let start = self.peek().clone();
        let mut visibility = Visibility::Private;

        // Take this path when self.match types(&[TokenType::Export]).
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

        // Take the branch when Rparen) is false.
        if !self.check(TokenType::Rparen) {
            // Run the loop body until it exits.
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

                // Take the branch when Comma]) is false.
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

    fn parse_extern_fn(&mut self) -> Result<spanda_ast::foundations::ExternFnDecl, SpandaError> {
        // Parse extern fn.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_extern_fn();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::{BridgeKind, ExternFnDecl, ModuleParamDecl};
        let start = self.previous().clone();
        let (library, bridge) = if self.match_types(&[TokenType::String]) {
            let TokenValue::String(lib) = self.previous().value.clone() else {
                return Err(SpandaError::Parse {
                    message: "Expected library name string after extern".into(),
                    line: self.previous().line,
                    column: self.previous().column,
                });
            };
            (Some(lib), BridgeKind::Native)
        } else if self.check(TokenType::Ident) {
            // Match on as str and handle each case.
            match self.peek().lexeme.as_str() {
                "python" => {
                    self.advance();
                    (Some("python".into()), BridgeKind::Python)
                }
                "cpp" => {
                    self.advance();
                    (Some("cpp".into()), BridgeKind::Cpp)
                }
                _ => (None, BridgeKind::Native),
            }
        } else {
            (None, BridgeKind::Native)
        };
        self.expect(TokenType::Fn, "Expected 'fn' in extern declaration")?;
        let name = self.parse_label("Expected extern function name")?;
        self.expect(TokenType::Lparen, "Expected '(' after extern function name")?;
        let mut params = Vec::new();

        // Take the branch when Rparen) is false.
        if !self.check(TokenType::Rparen) {
            // Run the loop body until it exits.
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

                // Take the branch when Comma]) is false.
                if !self.match_types(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.expect(TokenType::Rparen, "Expected ')' after extern parameters")?;
        self.expect(TokenType::Arrow, "Expected '->' after extern parameters")?;
        let return_type = self.parse_type_annotation()?;
        let end = self.expect(
            TokenType::Semicolon,
            "Expected ';' after extern declaration",
        )?;
        Ok(ExternFnDecl {
            name,
            library,
            bridge,
            params,
            return_type,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_test(&mut self) -> Result<spanda_ast::foundations::TestDecl, SpandaError> {
        // Parse test.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_test();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::TestDecl;
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
        // Parse struct.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_struct();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected struct name")?;
        let type_params = if self.check(TokenType::Lt) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };
        self.expect(TokenType::Lbrace, "Expected '{' after struct name")?;
        let mut fields = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let field_start = self.peek().clone();
            let field_name = self.expect(TokenType::Ident, "Expected field name")?;
            self.expect(TokenType::Colon, "Expected ':' after field name")?;
            let type_name = self.parse_type_name()?;
            self.expect(TokenType::Semicolon, "Expected ';' after field")?;
            fields.push(FieldDecl {
                name: field_name.lexeme,
                type_name,
                span: self.span_from(&field_start, self.previous()),
            });
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close struct")?;
        Ok(StructDecl::StructDecl {
            name: name.lexeme,
            type_params,
            fields,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_enum(&mut self) -> Result<EnumDecl, SpandaError> {
        // Parse enum.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_enum();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected enum name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after enum name")?;
        let mut variants = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let variant_start = self.peek().clone();
            let variant_name = self.expect(TokenType::Ident, "Expected enum variant")?;
            let mut field_types = Vec::new();

            // Take this path when self.match types(&[TokenType::Lparen]).
            if self.match_types(&[TokenType::Lparen]) {
                // Repeat while !self.check(TokenType::Rparen) && !self.check(TokenType::Eof).
                while !self.check(TokenType::Rparen) && !self.check(TokenType::Eof) {
                    field_types.push(self.parse_type_name()?);

                    // Take the branch when Comma]) is false.
                    if !self.match_types(&[TokenType::Comma]) {
                        break;
                    }
                }
                self.expect(TokenType::Rparen, "Expected ')' after enum variant fields")?;
            }
            variants.push(EnumVariantDecl {
                name: variant_name.lexeme,
                field_types,
                span: self.span_from(&variant_start, self.previous()),
            });

            // Take this path when self.match types(&[TokenType::Comma]).
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
        // Parse trait.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_trait();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected trait name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after trait name")?;
        let mut methods = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
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
        // Parse trait method.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_trait_method();

        // Compute start for the following logic.
        let start = self.advance(); // fn
        let name = self.parse_label("Expected method name after fn")?;
        self.expect(TokenType::Lparen, "Expected '(' after method name")?;
        let mut params = Vec::new();

        // Take the branch when Rparen) is false.
        if !self.check(TokenType::Rparen) {
            // Run the loop body until it exits.
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

                // Take the branch when Comma]) is false.
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
        //
        // Parameters:
        // - `self` — method receiver
        // - `kw` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_robot_member_keyword(kw);

        // Call check on the current instance.
        self.check(TokenType::Ident) && self.peek().lexeme == kw
    }

    fn parse_robot(&mut self) -> Result<RobotDecl, SpandaError> {
        // Parse robot.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_robot();

        // Compute start for the following logic.
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
        let mut pipelines = Vec::new();
        let mut watchdogs = Vec::new();
        let mut modes = Vec::new();
        let mut retries = Vec::new();
        let mut recovers = Vec::new();
        let fault_handlers = Vec::new();
        let mut state_machines = Vec::new();
        let mut events = Vec::new();
        let mut event_handlers = Vec::new();
        let mut trigger_handlers = Vec::new();
        let mut twin = None;
        let mut verify = None;
        let mut observe = None;
        let mut world_model = None;
        let mut identity = None;
        let mut audit = None;
        let mut provenance = None;
        let mut signed_records = Vec::new();
        let mut secrets = Vec::new();
        let mut trust = None;
        let mut permissions = None;
        let mut secure_comm = None;
        let mut trust_boundaries = Vec::new();
        let mut trait_impls = Vec::new();
        let mut requires_hardware = None;
        let mut requires_network = None;
        let mut requires_connectivity = None;
        let mut bluetooth = None;
        let mut mission = None;
        let mut buses = Vec::new();
        let mut peer_robots = Vec::new();
        let mut devices = Vec::new();
        let mut agent_channels = Vec::new();
        let mut twin_sync = None;

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.check(TokenType::Soc).
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
                // Take this path when self.is agent shorthand().
                if self.is_agent_shorthand() {
                    self.parse_agent_shorthand(&mut agents)?;
                } else {
                    agents.push(self.parse_agent()?);
                }
            } else if self.check(TokenType::Behavior) {
                behaviors.push(self.parse_behavior()?);
            } else if self.check(TokenType::Task) {
                tasks.push(self.parse_task()?);
            } else if self.check(TokenType::Ident) && self.peek().lexeme == "mode" {
                modes.push(self.parse_mode()?);
            } else if self.check(TokenType::Pipeline) {
                pipelines.push(self.parse_pipeline()?);
            } else if self.check(TokenType::Watchdog) {
                watchdogs.push(self.parse_watchdog()?);
            } else if self.check(TokenType::Retry) {
                retries.push(self.parse_retry()?);
            } else if self.check(TokenType::Recover) {
                recovers.push(self.parse_recover()?);
            } else if self.check(TokenType::StateMachine) {
                state_machines.push(self.parse_state_machine()?);
            } else if self.check(TokenType::Event) {
                events.push(self.parse_event()?);
            } else if self.check(TokenType::On) {
                let trigger = self.parse_on_trigger()?;

                // Take this path when let TriggerHandlerDecl::TriggerHandlerDecl.
                if let TriggerHandlerDecl::TriggerHandlerDecl {
                    trigger_kind: TriggerKind::Event { name },
                    body,
                    span,
                    ..
                } = &trigger
                {
                    event_handlers.push(EventHandlerDecl::EventHandlerDecl {
                        event_name: name.clone(),
                        body: body.clone(),
                        span: *span,
                    });
                }
                trigger_handlers.push(trigger);
            } else if self.check(TokenType::Every) {
                trigger_handlers.push(self.parse_every_trigger()?);
            } else if self.check(TokenType::When) {
                trigger_handlers.push(self.parse_when_trigger()?);
            } else if self.check(TokenType::While) {
                trigger_handlers.push(self.parse_while_trigger()?);
            } else if self.check(TokenType::Twin) && self.is_twin_sync() {
                twin_sync = Some(self.parse_twin_sync()?);
            } else if self.check(TokenType::Twin) {
                twin = Some(self.parse_twin()?);
            } else if self.check(TokenType::Verify) {
                verify = Some(self.parse_verify()?);
            } else if self.check(TokenType::Observe) {
                observe = Some(self.parse_observe()?);
            } else if self.is_robot_member_keyword("world_model") {
                world_model = Some(self.parse_world_model()?);
            } else if self.is_robot_member_keyword("secure_comm") {
                secure_comm = Some(self.parse_secure_comm_policy()?);
            } else if self.is_robot_member_keyword("trust_boundary") {
                trust_boundaries.push(self.parse_trust_boundary()?);
            } else if self.is_robot_member_keyword("secrets") {
                secrets.extend(self.parse_secrets_block()?);
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
            } else if self.check(TokenType::RequiresConnectivity) {
                requires_connectivity = Some(self.parse_requires_connectivity()?);
            } else if self.check(TokenType::Bluetooth) {
                bluetooth = Some(self.parse_bluetooth_config()?);
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
            pipelines,
            watchdogs,
            modes,
            retries,
            recovers,
            fault_handlers,
            state_machines,
            events,
            event_handlers,
            trigger_handlers,
            twin,
            verify,
            observe,
            world_model,
            identity,
            audit,
            provenance,
            signed_records,
            secrets,
            trust,
            permissions,
            secure_comm,
            trust_boundaries,
            requires_hardware,
            requires_network,
            requires_connectivity,
            bluetooth,
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
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_agent_shorthand();

        // Create mutable idx for accumulating results.
        let mut idx = self.pos + 1;

        // Take this path when idx >= self.tokens.len().
        if idx >= self.tokens.len() {
            return false;
        }

        // Take the branch when token type differs from Ident.
        if self.tokens[idx].token_type != TokenType::Ident {
            return false;
        }
        idx += 1;
        idx < self.tokens.len() && self.tokens[idx].token_type == TokenType::Semicolon
    }

    fn is_agent_channel(&self) -> bool {
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_agent_channel();

        // Compute idx for the following logic.
        let idx = self.pos;
        idx + 2 < self.tokens.len()
            && self.tokens[idx].token_type == TokenType::Ident
            && self.tokens[idx + 1].token_type == TokenType::Arrow
            && self.tokens[idx + 2].token_type == TokenType::Ident
    }

    fn parse_agent_shorthand(&mut self, agents: &mut Vec<AgentDecl>) -> Result<(), SpandaError> {
        // Parse agent shorthand.
        //
        // Parameters:
        // - `self` — method receiver
        // - `agents` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_agent_shorthand(agents);

        // Compute start for the following logic.
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
            trigger_handlers: Vec::new(),
            span: self.span_from(&start, self.previous()),
        });
        Ok(())
    }

    fn parse_bus(&mut self) -> Result<spanda_ast::comm_decl::BusDecl, SpandaError> {
        use spanda_ast::comm_decl::{BusDecl, TransportKind};
        let start = self.advance();
        let bus_name = self.expect(TokenType::Ident, "Expected bus name")?;

        if self.check(TokenType::Lbrace) {
            self.advance();
            let mut transport_name = bus_name.lexeme.clone();
            let mut broker_url = None;
            let mut encryption = None;
            let mut authentication = None;
            let mut integrity = None;

            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                let key = self.parse_config_key_token()?;
                self.expect(TokenType::Colon, "Expected ':' in bus field")?;
                match key.as_str() {
                    "transport" => {
                        transport_name = self.parse_config_value_string()?;
                    }
                    "url" => {
                        broker_url = Some(self.parse_config_value_string()?);
                    }
                    "encryption" => {
                        encryption = Some(
                            self.expect(TokenType::Ident, "Expected encryption mode")?
                                .lexeme,
                        );
                    }
                    "authentication" => {
                        authentication = Some(
                            self.expect(TokenType::Ident, "Expected authentication mode")?
                                .lexeme,
                        );
                    }
                    "integrity" => {
                        integrity = Some(
                            self.expect(TokenType::Ident, "Expected integrity mode")?
                                .lexeme,
                        );
                    }
                    other => {
                        return Err(SpandaError::Parse {
                            message: format!("Unknown bus field '{other}'"),
                            line: self.previous().line,
                            column: self.previous().column,
                        });
                    }
                }
                self.expect(TokenType::Semicolon, "Expected ';' after bus field")?;
            }
            let end = self.expect(TokenType::Rbrace, "Expected '}' to close bus block")?;
            let transport =
                TransportKind::from_ident(&transport_name).unwrap_or(TransportKind::Local);
            let _ = self.match_types(&[TokenType::Semicolon]);
            Ok(BusDecl::BusDecl {
                name: bus_name.lexeme,
                transport,
                transport_name: Some(transport_name),
                broker_url,
                encryption,
                authentication,
                integrity,
                span: self.span_from(&start, &end),
            })
        } else {
            self.expect(TokenType::Semicolon, "Expected ';' after bus declaration")?;
            let transport =
                TransportKind::from_ident(&bus_name.lexeme).unwrap_or(TransportKind::Local);
            Ok(BusDecl::BusDecl {
                name: bus_name.lexeme.clone(),
                transport,
                transport_name: None,
                broker_url: None,
                encryption: None,
                authentication: None,
                integrity: None,
                span: self.span_from(&start, self.previous()),
            })
        }
    }

    fn parse_peer_robot(&mut self) -> Result<spanda_ast::comm_decl::PeerRobotDecl, SpandaError> {
        // Parse peer robot.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_peer_robot();

        // Import the items needed by the logic below.
        use spanda_ast::comm_decl::PeerRobotDecl;
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected peer robot name")?;
        self.expect(TokenType::Semicolon, "Expected ';' after peer robot")?;
        Ok(PeerRobotDecl::PeerRobotDecl {
            name: name.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_device(&mut self) -> Result<spanda_ast::comm_decl::DeviceDecl, SpandaError> {
        // Parse device.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_device();

        // Import the items needed by the logic below.
        use spanda_ast::comm_decl::DeviceDecl;
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

    fn parse_agent_channel(
        &mut self,
    ) -> Result<spanda_ast::comm_decl::AgentChannelDecl, SpandaError> {
        // Parse agent channel.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_agent_channel();

        // Import the items needed by the logic below.
        use spanda_ast::comm_decl::AgentChannelDecl;
        let start = self.peek().clone();
        let from_agent = self
            .expect(TokenType::Ident, "Expected source agent")?
            .lexeme;
        self.expect(TokenType::Arrow, "Expected '->' in agent channel")?;
        let to_agent = self
            .expect(TokenType::Ident, "Expected target agent")?
            .lexeme;
        let message_type = if self.match_types(&[TokenType::Colon]) {
            self.expect(TokenType::Ident, "Expected message type after ':'")?
                .lexeme
        } else {
            String::new()
        };
        self.expect(TokenType::Semicolon, "Expected ';' after agent channel")?;
        Ok(AgentChannelDecl::AgentChannelDecl {
            from_agent,
            to_agent,
            message_type,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn is_twin_sync(&self) -> bool {
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_twin_sync();

        // Compute idx for the following logic.
        let idx = self.pos + 1;
        idx < self.tokens.len()
            && self.tokens[idx].token_type == TokenType::Ident
            && self.tokens[idx].lexeme == "sync"
    }

    #[allow(dead_code)]
    fn parse_twin_sync(&mut self) -> Result<spanda_ast::comm_decl::TwinSyncDecl, SpandaError> {
        // Parse twin sync.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_twin_sync();

        // Import the items needed by the logic below.
        use spanda_ast::comm_decl::TwinSyncDecl;
        let start = self.advance(); // twin
        self.expect(TokenType::Ident, "Expected 'sync' after twin")?;
        self.expect(TokenType::Lbrace, "Expected '{' after twin sync")?;
        let mut telemetry = false;
        let mut replay = false;
        let mut faults = false;
        let mut events = false;

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Telemetry]).
            if self.match_types(&[TokenType::Telemetry]) {
                telemetry = true;
                self.expect(TokenType::Semicolon, "Expected ';' after telemetry")?;
            } else if self.match_types(&[TokenType::Replay]) {
                replay = true;
                self.expect(TokenType::Semicolon, "Expected ';' after replay")?;
            } else if self.match_types(&[TokenType::Faults]) {
                faults = true;
                self.expect(TokenType::Semicolon, "Expected ';' after faults")?;
            } else if self.match_types(&[TokenType::Event])
                || (self.check(TokenType::Ident) && self.peek().lexeme == "events")
            {
                // Take this path when self.check(TokenType::Ident).
                if self.check(TokenType::Ident) {
                    self.advance();
                }
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

    fn parse_trait_impl(&mut self) -> Result<spanda_ast::foundations::TraitImplDecl, SpandaError> {
        // Parse trait impl.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_trait_impl();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::TraitImplDecl;
        let start = self.expect(TokenType::Impl, "Expected 'impl'")?;
        let trait_name = self.parse_label("Expected trait name after 'impl'")?;
        self.expect(TokenType::For, "Expected 'for' after trait name")?;
        let agent_name = self.parse_label("Expected agent name after 'for'")?;
        self.expect(TokenType::Lbrace, "Expected '{' after trait impl header")?;
        let mut methods = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
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
    ) -> Result<spanda_ast::foundations::TraitImplMethodDecl, SpandaError> {
        // Parse trait impl method.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_trait_impl_method();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::TraitImplMethodDecl;
        let start = self.expect(TokenType::Fn, "Expected 'fn' in trait impl method")?;
        let name = self.parse_label("Expected method name")?;
        self.expect(TokenType::Lparen, "Expected '(' after method name")?;
        let mut params = Vec::new();

        // Take the branch when Rparen) is false.
        if !self.check(TokenType::Rparen) {
            // Run the loop body until it exits.
            loop {
                let pstart = self.peek().clone();
                let pname = self.parse_label("Expected parameter name")?;
                self.expect(TokenType::Colon, "Expected ':' after parameter name")?;
                let ptype = self
                    .expect(TokenType::Ident, "Expected parameter type")?
                    .lexeme;
                params.push(spanda_ast::foundations::TraitParamDecl {
                    name: pname,
                    type_name: ptype,
                    span: self.span_from(&pstart, self.previous()),
                });

                // Take the branch when Comma]) is false.
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
        // Parse soc.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_soc();

        // Compute start for the following logic.
        let start = self.advance();
        let profile = self.expect(TokenType::Ident, "Expected SoC profile name")?;
        self.expect(TokenType::Semicolon, "Expected ';' after soc declaration")?;
        Ok(SocDecl::SocDecl {
            profile: profile.lexeme,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_hal(&mut self) -> Result<HalBlock, SpandaError> {
        // Parse hal.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_hal();

        // Compute start for the following logic.
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after hal")?;
        let mut members = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
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
        // Parse hal member.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_hal_member();

        // Compute start for the following logic.
        let start = self.peek().clone();

        // Take this path when self.match types(&[TokenType::I2c]).
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

        // Take this path when self.match types(&[TokenType::Spi]).
        if self.match_types(&[TokenType::Spi]) {
            let name = self.parse_hal_binding_name("Expected SPI bus name")?;
            self.expect(TokenType::At, "Expected 'at' after SPI bus name")?;
            let bus = self.expect(TokenType::Number, "Expected SPI bus number")?;
            let mut cs_pin = None;

            // Take this path when self.match types(&[TokenType::Pin]).
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

        // Take this path when self.match types(&[TokenType::Gpio]).
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

        // Take this path when self.match types(&[TokenType::Pwm]).
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

        // Take this path when self.match types(&[TokenType::Uart]).
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

        // Take this path when self.match types(&[TokenType::Adc]).
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
        // Parse frequency hz.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_frequency_hz();

        // Compute tok for the following logic.
        let tok = self.peek().clone();

        // Take the branch when token type equals Hz).
        if tok.token_type == TokenType::UnitLiteral && tok.unit == Some(UnitLexeme::Hz) {
            self.advance();
            return Ok(num(&tok));
        }

        // Take the branch when token type equals Number.
        if tok.token_type == TokenType::Number {
            self.advance();

            // Take the branch when lexeme equals "Hz".
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

    fn parse_message(&mut self) -> Result<spanda_ast::comm_decl::MessageDecl, SpandaError> {
        // Parse message.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_message();

        // Import the items needed by the logic below.
        use spanda_ast::comm_decl::MessageDecl;
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected message name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after message name")?;
        let mut fields = Vec::new();
        let mut version = None;

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take the branch when lexeme equals "version".
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

    fn parse_qos_block(&mut self) -> Result<spanda_ast::comm_decl::QosDecl, SpandaError> {
        // Parse qos block.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_qos_block();

        // Import the items needed by the logic below.
        use spanda_ast::comm_decl::{QosDecl, QosReliability};
        let start = self.peek().clone();
        self.expect(TokenType::Lbrace, "Expected '{' for topic QoS block")?;
        let mut reliability = None;
        let mut rate_hz = None;
        let mut deadline_ms = None;
        let mut history = None;

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Qos]).
            if self.match_types(&[TokenType::Qos]) {
                // Take this path when self.match types(&[TokenType::Reliable]).
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
        // Parse node.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_node();

        // Compute start for the following logic.
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
        // Parse topic.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_topic();

        // Import the items needed by the logic below.
        use spanda_ast::comm_decl::{TopicRole, TransportKind};
        let start = self.advance();
        let name = self.parse_label("Expected topic name")?;
        self.expect(TokenType::Colon, "Expected ':' after topic name")?;
        let message_type = if self.check(TokenType::Lbrace) {
            self.parse_label("Expected message type")?
        } else {
            self.parse_type_name_with_generics()?
        };
        let mut role = TopicRole::Both;
        let mut topic_path = None;
        let mut qos = None;
        let mut transport = None;
        let mut secure = None;

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
            let block_start = self.pos;
            self.advance();
            if self.check(TokenType::Secure) {
                secure = Some(self.parse_secure_block()?);
                self.expect(TokenType::Rbrace, "Expected '}' to close topic block")?;
                let _ = self.match_types(&[TokenType::Semicolon]);
            } else {
                self.pos = block_start;
                qos = Some(self.parse_qos_block()?);
            }
        }

        if self.match_types(&[TokenType::On]) && topic_path.is_none() && transport.is_none() {
            if self.check(TokenType::String) {
                topic_path = Some(str_val(&self.advance()));
            } else {
                let ident = self.expect(TokenType::Ident, "Expected transport name after on")?;
                transport = TransportKind::from_ident(&ident.lexeme);
            }
        }

        if secure.is_none() && self.check(TokenType::Secure) {
            secure = Some(self.parse_secure_block()?);
        }

        if secure.is_some() || qos.is_some() {
            let _ = self.match_types(&[TokenType::Semicolon]);
        } else {
            self.expect(TokenType::Semicolon, "Expected ';' after topic declaration")?;
        }
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
        // Parse service.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_service();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected service name")?;

        // Take this path when self.check(TokenType::Lbrace).
        if self.check(TokenType::Lbrace) {
            self.advance();
            let mut request_type = None;
            let mut response_type = None;
            let mut secure = None;

            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                if self.check(TokenType::Secure) {
                    secure = Some(self.parse_secure_block()?);
                } else if self.match_types(&[TokenType::Request]) {
                    request_type = Some(self.parse_type_name_with_generics()?);
                    self.expect(TokenType::Semicolon, "Expected ';' after request type")?;
                } else if self.match_types(&[TokenType::Response]) {
                    response_type = Some(self.parse_type_name_with_generics()?);
                    self.expect(TokenType::Semicolon, "Expected ';' after response type")?;
                } else {
                    let t = self.peek();
                    return Err(SpandaError::Parse {
                        message: "Expected request, response, or secure in service block".into(),
                        line: t.line,
                        column: t.column,
                    });
                }
            }
            let end = self.expect(TokenType::Rbrace, "Expected '}' to close service")?;
            let _ = self.match_types(&[TokenType::Semicolon]);
            return Ok(ServiceDecl::ServiceDecl {
                name: name.lexeme,
                service_type: None,
                request_type,
                response_type,
                secure,
                span: self.span_from(&start, &end),
            });
        }

        self.expect(TokenType::Colon, "Expected ':' after service name")?;
        let service_type = self.parse_type_name_with_generics()?;
        let mut secure = None;
        if self.check(TokenType::Lbrace) {
            self.advance();
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                if self.check(TokenType::Secure) {
                    secure = Some(self.parse_secure_block()?);
                } else {
                    let t = self.peek();
                    return Err(SpandaError::Parse {
                        message: "Expected secure block in service body".into(),
                        line: t.line,
                        column: t.column,
                    });
                }
            }
            self.expect(TokenType::Rbrace, "Expected '}' to close service block")?;
            let _ = self.match_types(&[TokenType::Semicolon]);
        }
        let secure = if secure.is_none() && self.check(TokenType::Secure) {
            Some(self.parse_secure_block()?)
        } else {
            secure
        };
        if self.previous().token_type != TokenType::Semicolon {
            let _ = self.match_types(&[TokenType::Semicolon]);
        }
        Ok(ServiceDecl::ServiceDecl {
            name: name.lexeme,
            service_type: Some(service_type),
            request_type: None,
            response_type: None,
            secure,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_action(&mut self) -> Result<ActionDecl, SpandaError> {
        // Parse action.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_action();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected action name")?;

        // Take this path when self.check(TokenType::Lbrace).
        if self.check(TokenType::Lbrace) {
            self.advance();
            let mut request_type = None;
            let mut feedback_type = None;
            let mut result_type = None;

            // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                // Take this path when self.match types(&[TokenType::Request]).
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
        // Parse sensor.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_sensor();

        // Compute start for the following logic.
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
            // Take this path when self.check(TokenType::String).
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
        // Parse actuator.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_actuator();

        // Compute start for the following logic.
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
        // Parse safety.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_safety();

        // Compute start for the following logic.
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after safety")?;
        let mut rules = Vec::new();
        let mut zones = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.check(TokenType::StopIf).
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

    fn parse_verify(&mut self) -> Result<spanda_ast::foundations::VerifyDecl, SpandaError> {
        // Parse verify.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_verify();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::VerifyDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after verify")?;
        let mut rules = Vec::new();
        let mut warnings = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Warning]).
            if self.match_types(&[TokenType::Warning]) {
                warnings.push(self.parse_expr()?);
            } else {
                rules.push(self.parse_expr()?);
            }
            self.expect(TokenType::Semicolon, "Expected ';' after verify rule")?;
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close verify block")?;
        Ok(VerifyDecl::VerifyDecl {
            rules,
            warnings,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_observe(&mut self) -> Result<spanda_ast::foundations::ObserveDecl, SpandaError> {
        // Parse observe.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_observe();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::ObserveDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after observe")?;
        let mut sensors = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
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

    fn parse_world_model(
        &mut self,
    ) -> Result<spanda_ast::foundations::WorldModelDecl, SpandaError> {
        // Parse world_model block on a robot declaration.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Parsed world-model declaration.
        //
        // Options:
        // Empty blocks default to enabled.
        //
        // Example:
        // let result = instance.parse_world_model();

        use spanda_ast::foundations::WorldModelDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after world_model")?;
        let mut enabled = true;

        // Read optional `enabled;` or `disabled;` flags inside the block.
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let flag = self.expect(TokenType::Ident, "Expected world_model flag")?;
            self.expect(TokenType::Semicolon, "Expected ';' after world_model flag")?;
            match flag.lexeme.as_str() {
                "enabled" => enabled = true,
                "disabled" => enabled = false,
                other => {
                    return Err(SpandaError::Parse {
                        message: format!("Unknown world_model flag '{other}' (use enabled or disabled)"),
                        line: flag.line,
                        column: flag.column,
                    })
                }
            }
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close world_model block")?;
        Ok(WorldModelDecl::WorldModelDecl {
            enabled,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_identity(&mut self) -> Result<IdentityDecl, SpandaError> {
        // Parse identity.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_identity();

        // Compute start for the following logic.
        let start = self.advance();
        let type_name = self.expect(TokenType::Ident, "Expected identity type name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after identity type")?;
        let mut fields = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let key = self.parse_config_key_token()?;
            self.expect(TokenType::Colon, "Expected ':' in identity field")?;
            let value = self.parse_config_value_string()?;
            self.expect(TokenType::Semicolon, "Expected ';' after identity field")?;
            fields.push((key, value));
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close identity")?;
        let _ = self.match_types(&[TokenType::Semicolon]);
        Ok(IdentityDecl::IdentityDecl {
            type_name: type_name.lexeme,
            fields,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_audit(&mut self) -> Result<AuditDecl, SpandaError> {
        // Parse audit.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_audit();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected audit name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after audit name")?;
        let mut records = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            self.expect(TokenType::Ident, "Expected 'record' in audit block")?;

            // Take the branch when lexeme differs from "record".
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
        // Parse provenance.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_provenance();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected provenance name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after provenance name")?;
        let mut hash_algo = "sha256".to_string();
        let mut signed_by = String::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let key = self.parse_config_key_token()?;
            self.expect(TokenType::Colon, "Expected ':' in provenance field")?;

            // Match on as str and handle each case.
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
        // Parse signed record.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_signed_record();

        // Compute start for the following logic.
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

    fn parse_secret(&mut self) -> Result<spanda_ast::foundations::SecretDecl, SpandaError> {
        // Parse secret.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_secret();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::{SecretDecl, SecretSourceDecl};
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected secret name")?;
        self.expect(TokenType::From, "Expected 'from' after secret name")?;
        let source = if self.match_types(&[TokenType::Env]) {
            if self.check(TokenType::Lparen) {
                self.expect(TokenType::Lparen, "Expected '(' after env")?;
                let var = str_val(&self.expect(TokenType::String, "Expected env var name")?);
                self.expect(TokenType::Rparen, "Expected ')' after env var")?;
                SecretSourceDecl::Env { var }
            } else {
                let var = str_val(&self.expect(TokenType::String, "Expected env var name")?);
                SecretSourceDecl::Env { var }
            }
        } else if self.match_types(&[TokenType::File]) {
            let path = self.parse_config_value_string()?;
            SecretSourceDecl::File { path }
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

    fn parse_trust(&mut self) -> Result<spanda_ast::foundations::TrustDecl, SpandaError> {
        // Parse trust.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_trust();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::TrustDecl;
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
        // Parse dotted capability.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_dotted_capability();

        // Compute first for the following logic.
        let first = if self.check(TokenType::Network) {
            self.advance();
            "network".to_string()
        } else if self.check(TokenType::Bluetooth) {
            self.advance();
            "bluetooth".to_string()
        } else if self.check(TokenType::Secret) {
            self.advance();
            "secret".to_string()
        } else if self.check(TokenType::Ident) {
            self.advance();
            self.previous().lexeme.clone()
        } else {
            return Err(SpandaError::Parse {
                message: "Expected capability name".into(),
                line: self.peek().line,
                column: self.peek().column,
            });
        };

        let mut cap = first;
        while self.match_types(&[TokenType::Dot]) {
            cap = format!("{cap}.{}", self.parse_capability_suffix()?);
        }
        Ok(cap)
    }

    fn parse_capability_suffix(&mut self) -> Result<String, SpandaError> {
        if self.check(TokenType::Ident) && self.peek().lexeme == "scan" {
            self.advance();
            Ok("scan".to_string())
        } else if self.check(TokenType::Ident) && self.peek().lexeme == "pair" {
            self.advance();
            Ok("pair".to_string())
        } else if self.check(TokenType::Ident) && self.peek().lexeme == "connect" {
            self.advance();
            Ok("connect".to_string())
        } else if self.check(TokenType::Ident) && self.peek().lexeme == "status" {
            self.advance();
            Ok("status".to_string())
        } else if self.check(TokenType::Ident) && self.peek().lexeme == "failover" {
            self.advance();
            Ok("failover".to_string())
        } else if self.check(TokenType::Ident) && self.peek().lexeme == "read" {
            self.advance();
            Ok("read".to_string())
        } else if self.check(TokenType::Publish) {
            self.advance();
            Ok("publish".to_string())
        } else if self.check(TokenType::Subscribe) {
            self.advance();
            Ok("subscribe".to_string())
        } else {
            self.parse_label("Expected capability suffix")
        }
    }

    fn parse_permissions(
        &mut self,
    ) -> Result<spanda_ast::foundations::PermissionsDecl, SpandaError> {
        // Parse permissions.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_permissions();

        // Import the items needed by the logic below.
        use spanda_ast::foundations::PermissionsDecl;
        let start = self.advance();
        self.expect(TokenType::Lbracket, "Expected '[' after permissions")?;
        let mut capabilities = Vec::new();

        // Take the branch when Rbracket) is false.
        if !self.check(TokenType::Rbracket) {
            // Run the loop body until it exits.
            loop {
                capabilities.push(self.parse_dotted_capability()?);

                // Take the branch when Comma]) is false.
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

    fn parse_secure_block(
        &mut self,
    ) -> Result<spanda_ast::foundations::SecureBlockDecl, SpandaError> {
        use spanda_ast::foundations::SecureBlockDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after secure")?;
        let mut signed = false;
        let mut signed_required = false;
        let mut min_trust = None;
        let mut requires = Vec::new();
        let mut encryption = None;
        let mut authentication = None;
        let mut integrity = None;
        let mut trusted_sources = Vec::new();
        let mut reject_untrusted = false;

        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let field = self.parse_label("Expected secure field name")?;

            if field == "trusted_sources" {
                self.expect(TokenType::Lbracket, "Expected '[' after trusted_sources")?;
                if !self.check(TokenType::Rbracket) {
                    loop {
                        trusted_sources.push(self.parse_label("Expected trusted source name")?);
                        if !self.match_types(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.expect(TokenType::Rbracket, "Expected ']' after trusted_sources")?;
            } else if self.check(TokenType::Assign) {
                self.advance();
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
            } else if field == "reject_untrusted" {
                if self.match_types(&[TokenType::True]) {
                    reject_untrusted = true;
                } else {
                    self.expect(
                        TokenType::False,
                        "Expected true or false for reject_untrusted",
                    )?;
                    reject_untrusted = false;
                }
            } else {
                let value = self.parse_label("Expected secure field value")?;
                match field.as_str() {
                    "encryption" => encryption = Some(value),
                    "authentication" => authentication = Some(value),
                    "integrity" => integrity = Some(value),
                    "signed" => {
                        signed_required = value == "required";
                        signed = value == "required" || value == "true";
                    }
                    other => {
                        return Err(SpandaError::Parse {
                            message: format!("Unknown secure field '{other}'"),
                            line: self.previous().line,
                            column: self.previous().column,
                        });
                    }
                }
            }

            self.expect(TokenType::Semicolon, "Expected ';' after secure field")?;
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close secure block")?;
        Ok(SecureBlockDecl {
            signed: signed || signed_required,
            signed_required,
            min_trust,
            requires,
            encryption,
            authentication,
            integrity,
            trusted_sources,
            reject_untrusted,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_secure_comm_policy(
        &mut self,
    ) -> Result<spanda_ast::foundations::SecureCommPolicyDecl, SpandaError> {
        use spanda_ast::foundations::SecureCommPolicyDecl;
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after secure_comm")?;
        let mut encryption = None;
        let mut authentication = None;
        let mut integrity = None;

        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let key = self.parse_config_key_token()?;
            self.expect(TokenType::Colon, "Expected ':' in secure_comm field")?;
            let value = self
                .expect(TokenType::Ident, "Expected secure_comm field value")?
                .lexeme;
            match key.as_str() {
                "encryption" => encryption = Some(value),
                "authentication" => authentication = Some(value),
                "integrity" => integrity = Some(value),
                other => {
                    return Err(SpandaError::Parse {
                        message: format!("Unknown secure_comm field '{other}'"),
                        line: self.previous().line,
                        column: self.previous().column,
                    });
                }
            }
            self.expect(TokenType::Semicolon, "Expected ';' after secure_comm field")?;
        }
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close secure_comm")?;
        let _ = self.match_types(&[TokenType::Semicolon]);
        Ok(SecureCommPolicyDecl::SecureCommPolicyDecl {
            encryption,
            authentication,
            integrity,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_trust_boundary(
        &mut self,
    ) -> Result<spanda_ast::foundations::TrustBoundaryDecl, SpandaError> {
        use spanda_ast::foundations::TrustBoundaryDecl;
        let start = self.advance();
        let name = self
            .expect(TokenType::Ident, "Expected trust boundary name")?
            .lexeme;
        self.expect(
            TokenType::Semicolon,
            "Expected ';' after trust_boundary declaration",
        )?;
        Ok(TrustBoundaryDecl::TrustBoundaryDecl {
            name,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_secrets_block(
        &mut self,
    ) -> Result<Vec<spanda_ast::foundations::SecretDecl>, SpandaError> {
        use spanda_ast::foundations::{SecretDecl, SecretSourceDecl};
        let start = self.advance();
        self.expect(TokenType::Lbrace, "Expected '{' after secrets")?;
        let mut secrets = Vec::new();

        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            let name = self.expect(TokenType::Ident, "Expected secret name")?;
            self.expect(TokenType::From, "Expected 'from' after secret name")?;
            let source = if self.match_types(&[TokenType::Env]) {
                let var = str_val(&self.expect(TokenType::String, "Expected env var name")?);
                SecretSourceDecl::Env { var }
            } else if self.match_types(&[TokenType::File]) {
                let path = self.parse_config_value_string()?;
                SecretSourceDecl::File { path }
            } else if self.check(TokenType::String) {
                SecretSourceDecl::Literal {
                    value: str_val(&self.advance()),
                }
            } else {
                let t = self.peek();
                return Err(SpandaError::Parse {
                    message: "Expected env, file, or string literal for secret source".into(),
                    line: t.line,
                    column: t.column,
                });
            };
            self.expect(TokenType::Semicolon, "Expected ';' after secret")?;
            secrets.push(SecretDecl::SecretDecl {
                name: name.lexeme,
                source,
                span: self.span_from(&start, self.previous()),
            });
        }
        self.expect(TokenType::Rbrace, "Expected '}' to close secrets block")?;
        let _ = self.match_types(&[TokenType::Semicolon]);
        Ok(secrets)
    }

    fn parse_type_name_with_generics(&mut self) -> Result<String, SpandaError> {
        let base = self.parse_label("Expected type name")?;
        if self.match_types(&[TokenType::Lt]) {
            let mut inner = String::new();
            loop {
                if self.check(TokenType::Gt) {
                    self.advance();
                    break;
                }
                if !inner.is_empty() && self.match_types(&[TokenType::Comma]) {
                    inner.push(',');
                    inner.push(' ');
                }
                inner.push_str(&self.parse_label("Expected generic type parameter")?);
            }
            Ok(format!("{base}<{inner}>"))
        } else {
            Ok(base)
        }
    }

    fn parse_config_value_string(&mut self) -> Result<String, SpandaError> {
        // Parse config value string.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_config_value_string();

        // Compute tok for the following logic.
        let tok = self.advance();

        // Match on token type and handle each case.
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
        // Expr path string.
        //
        // Parameters:
        // - `expr` — input value
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::parser::expr_path_string(expr);

        // Match on expr and handle each case.
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
        // Parse ai model.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_ai_model();

        // Compute start for the following logic.
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
        // Parse ai config entries.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_ai_config_entries();

        // Create mutable entries for accumulating results.
        let mut entries = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
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
        // Parse config key token.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_config_key_token();

        // take this path when self.check(TokenType::Ident) || self.check(TokenType::Provider).
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
        // Parse config value.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_config_value();

        // take this path when self.match types(&[TokenType::String]).
        if self.match_types(&[TokenType::String]) {
            Ok(ConfigValue::String(str_val(self.previous())))
        } else if self.match_types(&[TokenType::True]) {
            Ok(ConfigValue::Bool(true))
        } else if self.match_types(&[TokenType::False]) {
            Ok(ConfigValue::Bool(false))
        } else if self.match_types(&[TokenType::Number, TokenType::UnitLiteral]) {
            let n = num(self.previous());

            // Take this path when self.check(TokenType::Ident).
            if self.check(TokenType::Ident) {
                let unit = self.peek().lexeme.as_str();
                let scaled = match unit {
                    "GB" | "Gb" => n * 1024.0,
                    "MB" | "Mb" => n,
                    "TOPS" | "tops" => n,
                    _ => n,
                };

                // Take the branch when unit equals "GB".
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
        // Parse agent.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_agent();

        // Compute start for the following logic.
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
        let mut trigger_handlers = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Uses]).
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

                // Take the branch when Rbracket) is false.
                if !self.check(TokenType::Rbracket) {
                    // Run the loop body until it exits.
                    loop {
                        tools.push(self.expect(TokenType::Ident, "Expected tool name")?.lexeme);

                        // Take the branch when Comma]) is false.
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

                // Take the branch when Rbracket) is false.
                if !self.check(TokenType::Rbracket) {
                    // Run the loop body until it exits.
                    loop {
                        capabilities.push(self.parse_capability()?);

                        // Take the branch when Comma]) is false.
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
            } else if self.check(TokenType::On) {
                trigger_handlers.push(self.parse_agent_on_trigger()?);
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
            trigger_handlers,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_capability(&mut self) -> Result<CapabilityDecl, SpandaError> {
        // Parse capability.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_capability();

        // Compute start for the following logic.
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
        // Parse safety zone.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_safety_zone();

        // Compute start for the following logic.
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
        // Parse max speed rule.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_max_speed_rule();

        // Compute start for the following logic.
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
        // Parse stop if rule.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_stop_if_rule();

        // Compute start for the following logic.
        let start = self.advance();
        let condition = self.parse_expr()?;
        self.expect(TokenType::Semicolon, "Expected ';' after stop_if rule")?;
        Ok(SafetyRule::StopIfRule {
            condition,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_contract_clauses(&mut self) -> Result<ContractClauses, SpandaError> {
        // Parse contract clauses.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_contract_clauses();

        // Create mutable requires for accumulating results.
        let mut requires = None;
        let mut ensures = None;
        let mut invariant = None;

        // Repeat while !self.check(TokenType::Lbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Lbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Requires]).
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
        // Parse behavior.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_behavior();

        // Compute start for the following logic.
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
        // Parse task.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_task();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected task name")?;
        let mut priority = spanda_ast::foundations::TaskPriority::Normal;
        let mut deadline_ms = None;
        let mut jitter_ms_max = None;
        let mut isolated = false;

        // Take this path when self.check(TokenType::Ident).
        if self.check(TokenType::Ident) {
            // Take this path when let Some(parsed priority) =.
            if let Some(parsed_priority) =
                spanda_ast::foundations::TaskPriority::from_ident(&self.peek().lexeme)
            {
                self.advance();
                priority = parsed_priority;
            }
        }

        // Accept explicit `priority critical` after the task name.
        if self.match_types(&[TokenType::Priority]) {
            let level = self.expect(TokenType::Ident, "Expected priority level")?;
            priority = spanda_ast::foundations::TaskPriority::from_ident(&level.lexeme)
                .ok_or_else(|| SpandaError::Parse {
                    message: format!(
                        "Invalid priority '{}'; use critical, high, normal, or low",
                        level.lexeme
                    ),
                    line: level.line,
                    column: level.column,
                })?;
        }
        let interval_ms = if self.match_types(&[TokenType::Every]) {
            self.parse_duration()?
        } else {
            10.0
        };

        // Parse optional declared deadline and jitter constraints.
        if self.match_types(&[TokenType::Deadline]) {
            deadline_ms = Some(self.parse_duration()?);
        }
        if self.match_types(&[TokenType::Jitter]) {
            self.expect(TokenType::Lte, "Expected '<=' after jitter")?;
            jitter_ms_max = Some(self.parse_duration()?);
        }
        if self.match_types(&[TokenType::Isolated]) {
            isolated = true;
        }

        // Critical tasks always receive the highest scheduler priority.
        if matches!(priority, spanda_ast::foundations::TaskPriority::Critical) {
            priority = spanda_ast::foundations::TaskPriority::Critical;
        }
        let (requires, ensures, invariant) = self.parse_contract_clauses()?;
        self.expect(TokenType::Lbrace, "Expected '{' after task signature")?;
        let mut budget = None;

        // Take this path when self.check(TokenType::Budget).
        if self.check(TokenType::Budget) {
            budget = Some(self.parse_budget()?);
        }
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close task")?;
        Ok(TaskDecl::TaskDecl {
            name: name.lexeme,
            priority,
            interval_ms,
            deadline_ms,
            jitter_ms_max,
            isolated,
            requires,
            ensures,
            invariant,
            budget,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_pipeline(&mut self) -> Result<PipelineDecl, SpandaError> {
        // Parse latency-budgeted pipeline declaration.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected pipeline name")?;
        self.expect(TokenType::Budget, "Expected 'budget' after pipeline name")?;
        let budget_ms = self.parse_duration()?;
        self.expect(TokenType::Lbrace, "Expected '{' after pipeline budget")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close pipeline")?;
        Ok(PipelineDecl::PipelineDecl {
            name: name.lexeme,
            budget_ms,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_watchdog(&mut self) -> Result<WatchdogDecl, SpandaError> {
        // Parse watchdog timeout handler.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected watchdog name")?;
        let mut target = None;
        if self.check(TokenType::Ident) && self.peek().lexeme != "timeout" {
            target = Some(self.advance().lexeme);
        }
        if self.check(TokenType::Ident) && self.peek().lexeme == "timeout" {
            self.advance();
        } else {
            let t = self.peek();
            return Err(SpandaError::Parse {
                message: "Expected 'timeout' in watchdog declaration".into(),
                line: t.line,
                column: t.column,
            });
        }
        let timeout_ms = self.parse_duration()?;
        self.expect(TokenType::Lbrace, "Expected '{' after watchdog timeout")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close watchdog")?;
        Ok(WatchdogDecl::WatchdogDecl {
            name: name.lexeme,
            target,
            timeout_ms,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_mode(&mut self) -> Result<ModeDecl, SpandaError> {
        // Parse operating mode declaration.
        let start = self.advance(); // mode
        let name = self.expect(TokenType::Ident, "Expected mode name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after mode name")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close mode")?;
        Ok(ModeDecl::ModeDecl {
            name: name.lexeme,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_retry(&mut self) -> Result<RetryDecl, SpandaError> {
        // Parse retry policy with optional fallback block.
        let start = self.advance();
        let attempts_tok = self.expect(TokenType::Number, "Expected retry attempt count")?;
        let attempts = match attempts_tok.value {
            TokenValue::Number(n) if n >= 1.0 => n as u32,
            _ => {
                return Err(SpandaError::Parse {
                    message: "Retry attempts must be a positive number".into(),
                    line: attempts_tok.line,
                    column: attempts_tok.column,
                })
            }
        };
        self.expect(TokenType::Times, "Expected 'times' after retry count")?;
        self.expect(
            TokenType::Backoff,
            "Expected 'backoff' in retry declaration",
        )?;
        let backoff_ms = self.parse_duration()?;
        self.expect(TokenType::Lbrace, "Expected '{' after retry backoff")?;
        let body = self.parse_block()?;
        self.expect(TokenType::Rbrace, "Expected '}' to close retry body")?;
        let mut fallback = Vec::new();
        if self.match_types(&[TokenType::Fallback]) {
            self.expect(TokenType::Lbrace, "Expected '{' after fallback")?;
            fallback = self.parse_block()?;
            self.expect(TokenType::Rbrace, "Expected '}' to close fallback")?;
        }
        Ok(RetryDecl::RetryDecl {
            attempts,
            backoff_ms,
            body,
            fallback,
            span: self.span_from(&start, self.previous()),
        })
    }

    fn parse_recover(&mut self) -> Result<RecoverDecl, SpandaError> {
        // Parse recovery handler for a named error type.
        let start = self.advance();
        self.expect(TokenType::From, "Expected 'from' after recover")?;
        let error_name = self.expect(TokenType::Ident, "Expected error name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after recover error")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close recover")?;
        Ok(RecoverDecl::RecoverDecl {
            error_name: error_name.lexeme,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_validate_rule(&mut self) -> Result<ValidateRuleDecl, SpandaError> {
        // Parse top-level validate rule with regex pattern.
        let start = self.advance(); // validate
        let name = self.expect(TokenType::Ident, "Expected validate rule name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after validate name")?;
        self.expect(TokenType::Ident, "Expected 'value' in validate rule")?;
        self.expect(TokenType::Matches, "Expected 'matches' in validate rule")?;
        let pattern = self.parse_regex_literal()?;
        self.expect(TokenType::Semicolon, "Expected ';' after validate pattern")?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close validate rule")?;
        Ok(ValidateRuleDecl::ValidateRuleDecl {
            name: name.lexeme,
            pattern,
            span: self.span_from(&start, &end),
        })
    }

    fn regex_from_token(&self, tok: &Token) -> Result<RegexPattern, SpandaError> {
        // Convert a regex literal token into structured pattern data.
        let raw = tok.lexeme.as_str();
        let body = raw
            .trim_start_matches('/')
            .rsplit_once('/')
            .map(|(pat, flags)| (pat.to_string(), flags.to_string()))
            .ok_or_else(|| SpandaError::Parse {
                message: format!("Malformed regex literal '{raw}'"),
                line: tok.line,
                column: tok.column,
            })?;
        Ok(RegexPattern {
            source: body.0,
            flags: body.1,
            span: self.span_from(tok, tok),
        })
    }

    fn parse_regex_literal(&mut self) -> Result<RegexPattern, SpandaError> {
        // Parse `/pattern/flags` regex literal token into structured pattern data.
        let tok = self.expect(TokenType::RegexLiteral, "Expected regex literal")?;
        self.regex_from_token(&tok)
    }

    fn parse_state_machine(&mut self) -> Result<StateMachineDecl, SpandaError> {
        // Parse state machine.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_state_machine();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected state machine name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after state machine name")?;
        let mut states = Vec::new();
        let mut transitions = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::State]).
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
        // Parse event.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_event();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected event name")?;
        let mut fields = Vec::new();

        // Take this path when self.check(TokenType::Lbrace).
        if self.check(TokenType::Lbrace) {
            self.advance();

            // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
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

    fn parse_on_trigger(&mut self) -> Result<TriggerHandlerDecl, SpandaError> {
        // Parse on trigger.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_on_trigger();

        // Compute start for the following logic.
        let start = self.advance(); // on
        let kind = if self.check(TokenType::Ident) && self.peek().lexeme == "log" {
            self.advance();
            self.expect(TokenType::Matches, "Expected 'matches' after log")?;
            let pattern = self.parse_regex_literal()?;
            TriggerKind::LogMatch { pattern }
        } else if self.match_types(&[TokenType::Message]) {
            self.expect(TokenType::Dot, "Expected '.' after message in trigger")?;
            let field_part = self
                .expect(TokenType::Ident, "Expected field name after message.")?
                .lexeme;
            let field = format!("message.{field_part}");
            self.expect(TokenType::Matches, "Expected 'matches' after message field")?;
            let pattern = self.parse_regex_literal()?;
            TriggerKind::MessageMatch { field, pattern }
        } else if self.check(TokenType::Ident) && self.peek().lexeme.contains('.') {
            let field = self.advance().lexeme.clone();
            self.expect(TokenType::Matches, "Expected 'matches' after message field")?;
            let pattern = self.parse_regex_literal()?;
            TriggerKind::MessageMatch { field, pattern }
        } else if self.match_types(&[TokenType::State]) {
            self.parse_state_trigger_kind()?
        } else if self.match_types(&[TokenType::Safety]) {
            let event = self
                .expect(TokenType::Ident, "Expected safety event name")?
                .lexeme;
            TriggerKind::Safety { event }
        } else if self.check(TokenType::Hardware) {
            self.advance();
            let event = self
                .expect(TokenType::Ident, "Expected hardware event name")?
                .lexeme;
            TriggerKind::Hardware { event }
        } else if self.match_types(&[TokenType::Ai]) {
            let event = self
                .expect(TokenType::Ident, "Expected AI event name")?
                .lexeme;
            TriggerKind::Ai { event }
        } else if self.check(TokenType::Ident) && self.peek().lexeme == "verification" {
            self.advance();
            let event = self
                .expect(TokenType::Ident, "Expected verification event name")?
                .lexeme;
            TriggerKind::Verification { event }
        } else if self.match_types(&[TokenType::Twin]) {
            let event = self
                .expect(TokenType::Ident, "Expected twin event name")?
                .lexeme;
            TriggerKind::Twin { event }
        } else if self.match_types(&[TokenType::Geofence]) {
            let name = self
                .expect(TokenType::Ident, "Expected geofence name")?
                .lexeme;
            let phase = if self.match_types(&[TokenType::Exited]) {
                "exited".to_string()
            } else if self.match_types(&[TokenType::Entered]) {
                "entered".to_string()
            } else {
                self.expect(TokenType::Ident, "Expected entered or exited")?
                    .lexeme
                    .to_ascii_lowercase()
            };
            TriggerKind::Geofence { name, phase }
        } else if self.check(TokenType::Ident)
            || self.check(TokenType::Bluetooth)
            || self.check(TokenType::Network)
        {
            let first = self.parse_trigger_domain()?;
            if self.match_types(&[TokenType::Dot]) {
                let event = self
                    .expect(TokenType::Ident, "Expected event after '.'")?
                    .lexeme;
                let domain = first.to_ascii_lowercase();
                let event_lower = event.to_ascii_lowercase();
                if matches!(
                    domain.as_str(),
                    "network" | "cellular" | "bluetooth" | "wifi"
                ) || (matches!(domain.as_str(), "gps" | "gnss")
                    && matches!(event_lower.as_str(), "lost" | "acquired"))
                {
                    TriggerKind::Connectivity {
                        domain,
                        event: event_lower,
                    }
                } else if matches!(event_lower.as_str(), "fix" | "reading" | "update") {
                    TriggerKind::SensorEvent {
                        sensor: first,
                        event: event_lower,
                    }
                } else {
                    TriggerKind::Connectivity {
                        domain,
                        event: event_lower,
                    }
                }
            } else {
                TriggerKind::Event { name: first }
            }
        } else {
            let t = self.peek();
            return Err(SpandaError::Parse {
                message: "Expected trigger target name".into(),
                line: t.line,
                column: t.column,
            });
        };
        let priority = self.parse_optional_trigger_priority()?;
        self.expect(TokenType::Lbrace, "Expected '{' after trigger signature")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close trigger handler")?;
        Ok(TriggerHandlerDecl::TriggerHandlerDecl {
            trigger_kind: kind,
            priority,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_state_trigger_kind(&mut self) -> Result<TriggerKind, SpandaError> {
        // Parse state trigger kind.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_state_trigger_kind();

        // Compute phase for the following logic.
        let phase = self.expect(TokenType::Ident, "Expected Entered or Exited")?;
        let phase_name = phase.lexeme.to_ascii_lowercase();

        // Take the branch when phase name equals "entered".
        if phase_name == "entered" {
            self.expect(TokenType::Lparen, "Expected '(' after Entered")?;
            let state = self.expect(TokenType::Ident, "Expected state name")?.lexeme;
            self.expect(TokenType::Rparen, "Expected ')' after state name")?;
            Ok(TriggerKind::StateEntered { state })
        } else if phase_name == "exited" {
            self.expect(TokenType::Lparen, "Expected '(' after Exited")?;
            let state = self.expect(TokenType::Ident, "Expected state name")?.lexeme;
            self.expect(TokenType::Rparen, "Expected ')' after state name")?;
            Ok(TriggerKind::StateExited { state })
        } else {
            Err(SpandaError::Parse {
                message: "Expected Entered(State) or Exited(State) after 'on state'".into(),
                line: phase.line,
                column: phase.column,
            })
        }
    }

    fn parse_optional_trigger_priority(&mut self) -> Result<TaskPriority, SpandaError> {
        // Parse optional trigger priority.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_optional_trigger_priority();

        // take this path when self.match types(&[TokenType::Priority]).
        if self.match_types(&[TokenType::Priority]) {
            let ident = self.expect(TokenType::Ident, "Expected trigger priority level")?;
            TaskPriority::from_ident(&ident.lexeme).ok_or_else(|| SpandaError::Parse {
                message: "Trigger priority must be critical, high, normal, or low".into(),
                line: ident.line,
                column: ident.column,
            })
        } else if self.check(TokenType::Ident) {
            // Emit output when lexeme) provides a priority.
            if let Some(priority) = TaskPriority::from_ident(&self.peek().lexeme) {
                self.advance();
                Ok(priority)
            } else {
                Ok(TaskPriority::Normal)
            }
        } else {
            Ok(TaskPriority::Normal)
        }
    }

    fn parse_every_trigger(&mut self) -> Result<TriggerHandlerDecl, SpandaError> {
        // Parse every trigger.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_every_trigger();

        // Compute start for the following logic.
        let start = self.advance(); // every
        let interval_ms = self.parse_duration()?;
        let priority = self.parse_optional_trigger_priority()?;
        self.expect(TokenType::Lbrace, "Expected '{' after timer interval")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close timer trigger")?;
        Ok(TriggerHandlerDecl::TriggerHandlerDecl {
            trigger_kind: TriggerKind::Timer { interval_ms },
            priority,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_when_trigger(&mut self) -> Result<TriggerHandlerDecl, SpandaError> {
        // Parse when trigger.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_when_trigger();

        // Compute start for the following logic.
        let start = self.advance(); // when
        let expr = self.parse_expr()?;
        let priority = self.parse_optional_trigger_priority()?;
        self.expect(TokenType::Lbrace, "Expected '{' after when condition")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close when trigger")?;
        Ok(TriggerHandlerDecl::TriggerHandlerDecl {
            trigger_kind: TriggerKind::Condition { expr, level: false },
            priority,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_while_trigger(&mut self) -> Result<TriggerHandlerDecl, SpandaError> {
        // Parse while trigger.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_while_trigger();

        // Compute start for the following logic.
        let start = self.advance(); // while
        let expr = self.parse_expr()?;
        let priority = self.parse_optional_trigger_priority()?;
        self.expect(TokenType::Lbrace, "Expected '{' after while condition")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close while trigger")?;
        Ok(TriggerHandlerDecl::TriggerHandlerDecl {
            trigger_kind: TriggerKind::Condition { expr, level: true },
            priority,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_agent_on_trigger(&mut self) -> Result<TriggerHandlerDecl, SpandaError> {
        // Parse agent on trigger.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_agent_on_trigger();

        // Compute start for the following logic.
        let start = self.advance(); // on
        let name = self
            .expect(TokenType::Ident, "Expected trigger target in agent block")?
            .lexeme;
        let priority = self.parse_optional_trigger_priority()?;
        self.expect(TokenType::Lbrace, "Expected '{' after agent trigger")?;
        let body = self.parse_block()?;
        let end = self.expect(TokenType::Rbrace, "Expected '}' to close agent trigger")?;
        Ok(TriggerHandlerDecl::TriggerHandlerDecl {
            trigger_kind: TriggerKind::Message { topic: name },
            priority,
            body,
            span: self.span_from(&start, &end),
        })
    }

    fn parse_twin(&mut self) -> Result<TwinDecl, SpandaError> {
        // Parse twin.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_twin();

        // Compute start for the following logic.
        let start = self.advance();
        let name = self.expect(TokenType::Ident, "Expected twin name")?;
        self.expect(TokenType::Lbrace, "Expected '{' after twin name")?;
        let mut mirrors = Vec::new();
        let mut replay = false;

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            // Take this path when self.match types(&[TokenType::Mirror]).
            if self.match_types(&[TokenType::Mirror]) {
                mirrors.push(self.parse_label("Expected mirror field")?);
                self.expect(TokenType::Semicolon, "Expected ';' after mirror")?;
            } else if self.match_types(&[TokenType::Replay]) {
                replay = self.match_types(&[TokenType::True]);

                // Take the branch when replay is false.
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
        // Parse block.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_block();

        // Create mutable stmts for accumulating results.
        let mut stmts = Vec::new();

        // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
        while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, SpandaError> {
        // Parse stmt.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_stmt();

        // Compute start for the following logic.
        let start = self.peek().clone();

        // Parse stop_all_actuators(); as a dedicated safety statement.
        if self.check(TokenType::Ident) && self.peek().lexeme == "stop_all_actuators" {
            self.advance();
            self.expect(TokenType::Lparen, "Expected '(' after stop_all_actuators")?;
            self.expect(TokenType::Rparen, "Expected ')' after stop_all_actuators")?;
            self.expect(
                TokenType::Semicolon,
                "Expected ';' after stop_all_actuators",
            )?;
            return Ok(Stmt::StopAllActuatorsStmt {
                span: self.span_from(&start, self.previous()),
            });
        }

        // Parse run_pipeline name; pipeline execution statements.
        if self.check(TokenType::Ident) && self.peek().lexeme == "run_pipeline" {
            self.advance();
            let name = self.expect(
                TokenType::Ident,
                "Expected pipeline name after run_pipeline",
            )?;
            self.expect(TokenType::Semicolon, "Expected ';' after run_pipeline")?;
            return Ok(Stmt::RunPipelineStmt {
                name: name.lexeme,
                span: self.span_from(&start, self.previous()),
            });
        }

        // Parse navigate { goal: "..."; } statement sugar over navigation.navigate().
        if self.check(TokenType::Ident) && self.peek().lexeme == "navigate" {
            self.advance();
            self.expect(TokenType::Lbrace, "Expected '{' after navigate")?;
            let mut goal = None;
            let mut linear = None;
            let mut angular = None;
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                let field_name = if self.check(TokenType::Goal) {
                    self.advance();
                    "goal".to_string()
                } else if self.check(TokenType::Ident) {
                    self.advance().lexeme
                } else {
                    let t = self.peek();
                    return Err(SpandaError::Parse {
                        message: "Expected field name in navigate block".into(),
                        line: t.line,
                        column: t.column,
                    });
                };
                self.expect(TokenType::Colon, "Expected ':' after navigate field")?;
                match field_name.as_str() {
                    "goal" => goal = Some(self.parse_expr()?),
                    "linear" => linear = Some(self.parse_expr()?),
                    "angular" => angular = Some(self.parse_expr()?),
                    other => {
                        return Err(SpandaError::Parse {
                            message: format!("Unknown navigate field '{other}'"),
                            line: self.previous().line,
                            column: self.previous().column,
                        });
                    }
                }
                self.expect(TokenType::Semicolon, "Expected ';' after navigate field")?;
            }
            let end = self.expect(TokenType::Rbrace, "Expected '}' to close navigate block")?;
            let Some(goal_expr) = goal else {
                return Err(SpandaError::Parse {
                    message: "navigate block requires goal: ...".into(),
                    line: start.line,
                    column: start.column,
                });
            };
            return Ok(Stmt::NavigateStmt {
                goal: Box::new(goal_expr),
                linear: linear.map(Box::new),
                angular: angular.map(Box::new),
                span: self.span_from(&start, &end),
            });
        }

        // Parse fallback resource selection statements.
        if self.match_types(&[TokenType::Use]) {
            let resource = self.expect(TokenType::Ident, "Expected fallback resource name")?;
            self.expect(TokenType::Semicolon, "Expected ';' after use statement")?;
            return Ok(Stmt::UseFallbackStmt {
                resource: resource.lexeme,
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Return]).
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

        // Take this path when self.match types(&[TokenType::Let]).
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

            // Take this path when type annotation.is none() && init.is none().
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

        // Take this path when self.match types(&[TokenType::If]).
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

        // Take this path when self.match types(&[TokenType::Loop]).
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

        // Take this path when self.match types(&[TokenType::Publish]).
        if self.match_types(&[TokenType::Publish]) {
            let topic_name = self.parse_subscribe_target()?;

            // Take this path when self.match types(&[TokenType::Lparen]).
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
            let mut filter = None;
            if self.match_types(&[TokenType::Where]) {
                let field = self.parse_dotted_name("Expected filter field after where")?;
                self.expect(TokenType::Matches, "Expected 'matches' in subscribe filter")?;
                let pattern = self.parse_regex_literal()?;
                filter = Some(SubscribeFilterDecl {
                    field,
                    pattern,
                    span: self.span_from(&start, self.previous()),
                });
            }
            self.expect(TokenType::Semicolon, "Expected ';' after subscribe")?;
            return Ok(Stmt::SubscribeStmt {
                target,
                filter,
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Execute]).
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

        // Take this path when self.match types(&[TokenType::Discover]).
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

        // Take this path when self.match types(&[TokenType::Receive]).
        if self.match_types(&[TokenType::Receive]) {
            let topic_name = self.parse_subscribe_target()?;
            self.expect(TokenType::To, "Expected 'to' after topic in receive")?;
            let var = self.expect(TokenType::Ident, "Expected variable name")?;
            self.expect(TokenType::Semicolon, "Expected ';' after receive")?;
            return Ok(Stmt::ReceiveStmt {
                topic_name,
                var_name: var.lexeme,
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Call]).
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

        // Take this path when self.match types(&[TokenType::SendGoal]).
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

        // Take this path when self.match types(&[TokenType::EmergencyStop]).
        if self.match_types(&[TokenType::EmergencyStop]) {
            self.expect(TokenType::Semicolon, "Expected ';' after emergency_stop")?;
            return Ok(Stmt::EmergencyStopStmt {
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::ResetEmergencyStop]).
        if self.match_types(&[TokenType::ResetEmergencyStop]) {
            self.expect(
                TokenType::Semicolon,
                "Expected ';' after reset_emergency_stop",
            )?;
            return Ok(Stmt::ResetEmergencyStopStmt {
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Emit]).
        if self.match_types(&[TokenType::Emit]) {
            let event = self.parse_label("Expected event name after emit")?;
            self.expect(TokenType::Semicolon, "Expected ';' after emit statement")?;
            return Ok(Stmt::EmitStmt {
                event_name: event,
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Enter]).
        if self.match_types(&[TokenType::Enter]) {
            let target = self.parse_label("Expected state or mode name after enter")?;
            self.expect(TokenType::Semicolon, "Expected ';' after enter statement")?;
            if target.ends_with("_mode")
                || matches!(target.as_str(), "normal" | "degraded" | "emergency")
            {
                let mode = target.strip_suffix("_mode").unwrap_or(&target).to_string();
                return Ok(Stmt::EnterModeStmt {
                    mode,
                    span: self.span_from(&start, self.previous()),
                });
            }
            return Ok(Stmt::EnterStmt {
                state_name: target,
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Remember]).
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

        // Take this path when self.match types(&[TokenType::Spawn]).
        if self.match_types(&[TokenType::Spawn]) {
            let callee = self.parse_label("Expected function name after spawn")?;
            let mut args = Vec::new();

            // Take this path when self.match types(&[TokenType::Lparen]).
            if self.match_types(&[TokenType::Lparen]) {
                // Take the branch when Rparen) is false.
                if !self.check(TokenType::Rparen) {
                    // Run the loop body until it exits.
                    loop {
                        args.push(self.parse_expr()?);

                        // Take the branch when Comma]) is false.
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

        // Take this path when self.match types(&[TokenType::Select]).
        if self.match_types(&[TokenType::Select]) {
            self.expect(TokenType::Lbrace, "Expected '{' after select")?;
            let mut arms = Vec::new();

            // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                let arm_start = self.peek().clone();
                let recv = self.expect(TokenType::Ident, "Expected 'recv' in select arm")?;

                // Take the branch when lexeme differs from "recv".
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
                arms.push(spanda_ast::foundations::SelectArm {
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

        // Take this path when self.match types(&[TokenType::Parallel]).
        if self.match_types(&[TokenType::Parallel]) {
            self.expect(TokenType::Lbrace, "Expected '{' after parallel")?;
            let body = self.parse_block()?;
            let end = self.expect(TokenType::Rbrace, "Expected '}' to close parallel")?;
            self.expect(TokenType::Semicolon, "Expected ';' after parallel block")?;
            return Ok(Stmt::ParallelStmt {
                body,
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
        // Parse subscribe target.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_subscribe_target();

        // Compute first for the following logic.
        let first = self.parse_label("Expected subscribe target")?;

        // Take this path when self.match types(&[TokenType::Dot]).
        if self.match_types(&[TokenType::Dot]) {
            let second = self.parse_label("Expected member after '.'")?;
            Ok(format!("{first}.{second}"))
        } else {
            Ok(first)
        }
    }

    fn parse_discover_target(
        &mut self,
    ) -> Result<spanda_ast::comm_decl::DiscoverTarget, SpandaError> {
        // Parse discover target.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_discover_target();

        // Import the items needed by the logic below.
        use spanda_ast::comm_decl::DiscoverTarget;
        let name = self
            .expect(TokenType::Ident, "Expected discover target")?
            .lexeme;

        // Match on as str and handle each case.
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
    ) -> Result<Option<spanda_ast::comm_decl::DiscoverFilter>, SpandaError> {
        // Parse discover filter.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_discover_filter();

        // take the branch when Where]) is false.
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
        Ok(Some(spanda_ast::comm_decl::DiscoverFilter {
            capability: Some(cap),
        }))
    }

    fn parse_duration(&mut self) -> Result<f64, SpandaError> {
        // Parse duration.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_duration();

        // Compute tok for the following logic.
        let tok = self.peek().clone();

        // Take the branch when token type equals Ms).
        if tok.token_type == TokenType::UnitLiteral && tok.unit == Some(UnitLexeme::Ms) {
            self.advance();
            return Ok(num(&tok));
        }

        // Take the branch when token type equals S).
        if tok.token_type == TokenType::UnitLiteral && tok.unit == Some(UnitLexeme::S) {
            self.advance();
            return Ok(num(&tok) * 1000.0);
        }

        // Take the branch when token type equals Number.
        if tok.token_type == TokenType::Number {
            self.advance();

            // Take the branch when lexeme equals "ms".
            if self.check(TokenType::Ident) && self.peek().lexeme == "ms" {
                self.advance();
            } else if self.check(TokenType::Ident) && self.peek().lexeme == "s" {
                self.advance();
                return Ok(num(&tok) * 1000.0);
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
        // Parse unit suffix.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_unit_suffix();

        // Call try parse unit suffix on the current instance.
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
        // Try parse unit suffix.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.try_parse_unit_suffix();

        // take this path when self.check(TokenType::UnitLiteral).
        if self.check(TokenType::UnitLiteral) {
            let t = self.advance();
            return Some(unit_from_lexeme(t.unit?));
        }

        // Take this path when self.check(TokenType::Ident).
        if self.check(TokenType::Ident)
            && self.peek().lexeme == "m"
            && self.tokens.get(self.pos + 1).map(|t| t.token_type) == Some(TokenType::Slash)
            && self.tokens.get(self.pos + 2).map(|t| t.lexeme.as_str()) == Some("s")
        {
            self.pos += 3;
            return Some(UnitKind::MPerS);
        }

        // Take this path when self.check(TokenType::Ident).
        if self.check(TokenType::Ident)
            && self.peek().lexeme == "rad"
            && self.tokens.get(self.pos + 1).map(|t| t.token_type) == Some(TokenType::Slash)
            && self.tokens.get(self.pos + 2).map(|t| t.lexeme.as_str()) == Some("s")
        {
            self.pos += 3;
            return Some(UnitKind::RadPerS);
        }

        // Take this path when self.check(TokenType::Ident).
        if self.check(TokenType::Ident) {
            let lexeme = self.peek().lexeme.clone();

            // Take this path when is unit ident(&lexeme).
            if is_unit_ident(&lexeme) {
                self.advance();
                return Some(UnitKind::from_lexeme(&lexeme));
            }
        }
        None
    }

    fn parse_expr(&mut self) -> Result<Expr, SpandaError> {
        // Parse expr.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_expr();

        // Call parse or on the current instance.
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, SpandaError> {
        // Parse or.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_or();

        // Create mutable left for accumulating results.
        let mut left = self.parse_and()?;

        // Repeat while self.match types(&[TokenType::Or]).
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
        // Parse and.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_and();

        // Create mutable left for accumulating results.
        let mut left = self.parse_comparison()?;

        // Repeat while self.match types(&[TokenType::And]).
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
        // Parse comparison.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_comparison();

        // Create mutable left for accumulating results.
        let mut left = self.parse_additive()?;

        // Repeat while self.match types(&[.
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
        // Parse additive.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_additive();

        // Create mutable left for accumulating results.
        let mut left = self.parse_multiplicative()?;

        // Repeat while self.match types(&[TokenType::Plus, TokenType::Minus]).
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
        // Parse multiplicative.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_multiplicative();

        // Create mutable left for accumulating results.
        let mut left = self.parse_unary()?;

        // Repeat while self.match types(&[TokenType::Star, TokenType::Slash]).
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
        // Parse unary.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_unary();

        // take this path when self.match types(&[TokenType::Await]).
        if self.match_types(&[TokenType::Await]) {
            let start = self.previous().clone();
            let operand = self.parse_unary()?;
            return Ok(Expr::AwaitExpr {
                operand: Box::new(operand),
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Spawn]).
        if self.match_types(&[TokenType::Spawn]) {
            let start = self.previous().clone();
            let callee = Box::new(Expr::IdentExpr {
                name: self.parse_label("Expected function name after spawn")?,
                span: self.span_from(&start, self.previous()),
            });
            let mut args = Vec::new();

            // Take this path when self.match types(&[TokenType::Lparen]).
            if self.match_types(&[TokenType::Lparen]) {
                // Take the branch when Rparen) is false.
                if !self.check(TokenType::Rparen) {
                    // Run the loop body until it exits.
                    loop {
                        args.push(self.parse_expr()?);

                        // Take the branch when Comma]) is false.
                        if !self.match_types(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.expect(TokenType::Rparen, "Expected ')' after spawn arguments")?;
            }
            return Ok(Expr::SpawnExpr {
                callee,
                args,
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Minus, TokenType::Not]).
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
        // Parse postfix.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_postfix();

        // Create mutable expr for accumulating results.
        let mut expr = self.parse_primary()?;

        // Run the loop body until it exits.
        loop {
            // Take this path when self.match types(&[TokenType::Dot]).
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

                // Take the branch when Rparen) is false.
                if !self.check(TokenType::Rparen) {
                    // Run the loop body until it exits.
                    loop {
                        // Take this path when self.is named arg start().
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

                        // Take the branch when Comma]) is false.
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
            } else if self.check(TokenType::Lt) {
                // Take this path when let Expr::IdentExpr { name, span, .. } = &expr.
                if let Expr::IdentExpr { name, span, .. } = &expr {
                    // Take this path when name.chars().next().is some and(|c| c.is uppercase()).
                    if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        let start = span.start;
                        let full = self.finish_generic_type_name(name.clone())?;
                        expr = Expr::IdentExpr {
                            name: full,
                            span: Span {
                                start,
                                end: loc(self.previous()),
                            },
                        };
                        continue;
                    }
                }
                break;
            } else if self.check(TokenType::Lbrace) {
                // Take this path when let Expr::IdentExpr { name, .. } = &expr.
                if let Expr::IdentExpr { name, .. } = &expr {
                    // Take this path when name.chars().next().is some and(|c| c.is uppercase()).
                    if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        self.advance();
                        let mut fields = Vec::new();

                        // Take the branch when Rbrace) is false.
                        if !self.check(TokenType::Rbrace) {
                            // Run the loop body until it exits.
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

                                // Take the branch when Comma]) is false.
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
        // Parse primary.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_primary();

        // Compute start for the following logic.
        let start = self.peek().clone();

        // Take this path when self.match types(&[TokenType::Match]).
        if self.match_types(&[TokenType::Match]) {
            let scrutinee = self.parse_expr()?;
            self.expect(TokenType::Lbrace, "Expected '{' after match scrutinee")?;
            let mut arms = Vec::new();

            // Repeat while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof).
            while !self.check(TokenType::Rbrace) && !self.check(TokenType::Eof) {
                let arm_start = self.peek().clone();
                let variant = self.parse_import_segment("Expected match arm variant")?;
                let mut bindings = Vec::new();

                // Take this path when self.match types(&[TokenType::Lparen]).
                if self.match_types(&[TokenType::Lparen]) {
                    // Repeat while !self.check(TokenType::Rparen) && !self.check(TokenType::Eof).
                    while !self.check(TokenType::Rparen) && !self.check(TokenType::Eof) {
                        bindings.push(self.parse_import_segment("Expected binding name")?);

                        // Take the branch when Comma]) is false.
                        if !self.match_types(&[TokenType::Comma]) {
                            break;
                        }
                    }
                    self.expect(TokenType::Rparen, "Expected ')' after match bindings")?;
                }
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
                    bindings,
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

        // Take this path when self.match types(&[TokenType::Call]).
        if self.match_types(&[TokenType::Call]) {
            let service = self.expect(TokenType::Ident, "Expected service name after call")?;
            self.expect(TokenType::Lparen, "Expected '(' after service name")?;
            self.expect(TokenType::Rparen, "Expected ')' after service arguments")?;
            return Ok(Expr::ServiceCallExpr {
                service_name: service.lexeme,
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Execute]).
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

        // Take this path when self.match types(&[TokenType::Discover]).
        if self.match_types(&[TokenType::Discover]) {
            let target = self.parse_discover_target()?;
            let filter = self.parse_discover_filter()?;
            return Ok(Expr::DiscoverExpr {
                target,
                filter,
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Robot]).
        if self.match_types(&[TokenType::Robot]) {
            return Ok(Expr::IdentExpr {
                name: "robot".into(),
                span: self.span_from(&start, self.previous()),
            });
        }

        // Treat program-level fleet/mission keywords as runtime object references.
        if self.match_types(&[TokenType::Fleet]) {
            return Ok(Expr::IdentExpr {
                name: "fleet".into(),
                span: self.span_from(&start, self.previous()),
            });
        }
        if self.match_types(&[TokenType::Mission]) {
            return Ok(Expr::IdentExpr {
                name: "mission".into(),
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Safety]).
        if self.match_types(&[TokenType::Safety]) {
            return Ok(Expr::IdentExpr {
                name: "safety".into(),
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::Actuator]).
        if self.match_types(&[TokenType::Actuator]) {
            return Ok(Expr::IdentExpr {
                name: "actuator".into(),
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::True]).
        if self.match_types(&[TokenType::True]) {
            return Ok(Expr::LiteralExpr {
                value: LiteralValue::Bool(true),
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::False]).
        if self.match_types(&[TokenType::False]) {
            return Ok(Expr::LiteralExpr {
                value: LiteralValue::Bool(false),
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[TokenType::RegexLiteral]).
        if self.match_types(&[TokenType::RegexLiteral]) {
            let tok = self.previous().clone();
            return Ok(Expr::LiteralExpr {
                value: LiteralValue::Regex(self.regex_from_token(&tok)?),
                span: self.span_from(&tok, &tok),
            });
        }

        // Take this path when self.match types(&[TokenType::Number]).
        if self.match_types(&[TokenType::Number]) {
            let tok = self.previous().clone();

            // Emit output when try parse unit suffix provides a unit.
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

        // Take this path when self.match types(&[TokenType::UnitLiteral]).
        if self.match_types(&[TokenType::UnitLiteral]) {
            let tok = self.previous().clone();
            return Ok(Expr::UnitLiteralExpr {
                value: num(&tok),
                unit: unit_from_lexeme(tok.unit.unwrap()),
                span: self.span_from(&start, &tok),
            });
        }

        // Take this path when self.match types(&[TokenType::String]).
        if self.match_types(&[TokenType::String]) {
            return Ok(Expr::LiteralExpr {
                value: LiteralValue::String(str_val(self.previous())),
                span: self.span_from(&start, self.previous()),
            });
        }

        // Take this path when self.match types(&[.
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

        // Take this path when self.match types(&[TokenType::Lparen]).
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
        // Parse property name.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_property_name();

        // Allow common method names that overlap with reliability keywords.
        if self.match_types(&[TokenType::Matches]) {
            let t = self.previous().clone();
            return Ok(Token {
                token_type: TokenType::Ident,
                lexeme: "matches".into(),
                value: TokenValue::Null,
                unit: None,
                line: t.line,
                column: t.column,
                offset: t.offset,
            });
        }
        if self.check(TokenType::Ident) && self.peek().lexeme == "validate" {
            let t = self.advance().clone();
            return Ok(Token {
                token_type: TokenType::Ident,
                lexeme: "validate".into(),
                value: TokenValue::Null,
                unit: None,
                line: t.line,
                column: t.column,
                offset: t.offset,
            });
        }

        // Compute lexeme for the following logic.
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
        // Parse local name.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_local_name(message);

        // Compute lexeme for the following logic.
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
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_named_arg_start();

        // Transform self and continue the chain.
        self.tokens.get(self.pos + 1).map(|t| t.token_type) == Some(TokenType::Colon)
            && (self.check(TokenType::Ident)
                || self.check(TokenType::From)
                || self.check(TokenType::To)
                || self.check(TokenType::Goal))
    }

    fn parse_named_arg_name(&mut self) -> Result<String, SpandaError> {
        // Parse named arg name.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_named_arg_name();

        // take this path when self.match types(&[TokenType::From]).
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
    // Loc.
    //
    // Parameters:
    // - `t` — input value
    //
    // Returns:
    // SourceLocation.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::parser::loc(t);

    // Produce SourceLocation as the result.
    SourceLocation {
        line: t.line,
        column: t.column,
        offset: t.offset,
    }
}

fn num(tok: &Token) -> f64 {
    // Num.
    //
    // Parameters:
    // - `tok` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::parser::num(tok);

    // Match on value and handle each case.
    match &tok.value {
        TokenValue::Number(n) => *n,
        _ => 0.0,
    }
}

fn str_val(tok: &Token) -> String {
    // Str val.
    //
    // Parameters:
    // - `tok` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::parser::str_val(tok);

    // Match on value and handle each case.
    match &tok.value {
        TokenValue::String(s) => s.clone(),
        _ => tok.lexeme.clone(),
    }
}

fn is_unit_ident(lexeme: &str) -> bool {
    //
    // Parameters:
    // - `lexeme` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::parser::is_unit_ident(lexeme);

    // Produce matches! as the result.
    matches!(lexeme, "m" | "s" | "ms" | "rad" | "deg" | "Hz")
}

fn expr_span(expr: &Expr) -> Span {
    // Expr span.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Span.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::parser::expr_span(expr);

    // Match on expr and handle each case.
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
        Expr::SpawnExpr { span, .. } => *span,
    }
}

fn re_span_expr(expr: Expr, span: Span) -> Expr {
    // Re span expr.
    //
    // Parameters:
    // - `expr` — input value
    // - `span` — input value
    //
    // Returns:
    // Expr.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::parser::re_span_expr(expr, span);

    // Match on expr and handle each case.
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
        Expr::SpawnExpr { callee, args, span } => Expr::SpawnExpr {
            callee: Box::new((*callee).clone()),
            args: args.clone(),
            span,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_lexer::tokenize;

    #[test]
    fn parses_complete_robot() {
        // Parses complete robot.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::parser::parses_complete_robot();

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
        // Parses loop every.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::parser::parses_loop_every();

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
        // Parses if else.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::parser::parses_if_else();

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
        // Parses max speed rule.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::parser::parses_max_speed_rule();

        let ast = parse(tokenize("robot R { safety { max_speed = 1.5 m/s; } }").unwrap()).unwrap();
        let RobotDecl::RobotDecl { safety, .. } = &ast.robots()[0];
        let Some(s) = safety else {
            panic!("expected safety block");
        };
        let SafetyBlock::SafetyBlock { rules, .. } = s;
        assert!(matches!(rules[0], SafetyRule::MaxSpeedRule { .. }));
    }

    #[test]
    fn parses_world_model_after_observe() {
        use spanda_ast::foundations::WorldModelDecl;
        let source = r#"robot R {
  bus sim;
  sensor lidar: Lidar on "/scan";
  observe { lidar; }
  world_model { enabled; }
  safety { max_speed = 1.0 m/s; }
  behavior b() {}
}"#;
        let ast = parse(tokenize(source).unwrap()).unwrap();
        let RobotDecl::RobotDecl { world_model, observe, .. } = &ast.robots()[0];
        assert!(observe.is_some());
        let Some(WorldModelDecl::WorldModelDecl { enabled, .. }) = world_model else {
            panic!("expected world_model block");
        };
        assert!(*enabled);
    }
}
