//! pretty support for Spanda.
//!
use crate::ast::*;
use crate::comm::{DiscoverFilter, DiscoverTarget};
use crate::foundations::{CapabilityDecl, Visibility};

struct PrettyPrinter {
    out: String,
    indent: usize,
    at_line_start: bool,
}

impl PrettyPrinter {
    fn new() -> Self {
        // Create a new instance.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::pretty::new();

        // Assemble the struct fields and return it.
        Self {
            out: String::new(),
            indent: 0,
            at_line_start: true,
        }
    }

    fn finish(mut self) -> String {
        // Finish.
        //
        // Parameters:
        // - `mut self` — input value
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::pretty::finish(mut self);

        // Repeat while self.out.ends with('\n').
        while self.out.ends_with('\n') {
            self.out.pop();
        }
        self.out.push('\n');
        self.out
    }

    fn write_indent(&mut self) {
        // Write indent.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.write_indent();

        // take this path when self.at line start.
        if self.at_line_start {
            // Iterate over indent.
            for _ in 0..self.indent {
                self.out.push_str("  ");
            }
            self.at_line_start = false;
        }
    }

    fn write(&mut self, text: &str) {
        // Write.
        //
        // Parameters:
        // - `self` — method receiver
        // - `text` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.write(text);

        // Call write indent on the current instance.
        self.write_indent();
        self.out.push_str(text);
    }

    fn space(&mut self) {
        // Space.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.space();

        // take the branch when ends with is false.
        if !self.at_line_start && !self.out.ends_with(' ') && !self.out.ends_with('\n') {
            self.out.push(' ');
        }
    }

    fn newline(&mut self) {
        // Newline.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.newline();

        // Append into self.
        self.out.push('\n');
        self.at_line_start = true;
    }

    fn write_line(&mut self, text: &str) {
        // Write line.
        //
        // Parameters:
        // - `self` — method receiver
        // - `text` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.write_line(text);

        // Call write on the current instance.
        self.write(text);
        self.newline();
    }

    fn open_block(&mut self, header: &str) {
        // Open block.
        //
        // Parameters:
        // - `self` — method receiver
        // - `header` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.open_block(header);

        // Call write line on the current instance.
        self.write_line(&format!("{header} {{"));
        self.indent += 1;
    }

    fn close_block(&mut self, suffix: &str) {
        // Close block.
        //
        // Parameters:
        // - `self` — method receiver
        // - `suffix` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.close_block(suffix);

        // Call saturating sub on the current instance.
        self.indent = self.indent.saturating_sub(1);
        self.write_line(&format!("}}{suffix}"));
    }

    fn emit_source_span(&mut self, source: &str, span: &Span) {
        // Emit source span.
        //
        // Parameters:
        // - `self` — method receiver
        // - `source` — input value
        // - `span` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.emit_source_span(source, span);

        // Compute Some for the following logic.
        let Some(chunk) = source.get(span.start.offset..span.end.offset) else {
            return;
        };

        // Handle each input line.
        for line in chunk.lines() {
            self.write_line(line.trim_end());
        }
    }

    fn print_type(&mut self, ty: &SpandaType) {
        // Print type.
        //
        // Parameters:
        // - `self` — method receiver
        // - `ty` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_type(ty);

        // Match on ty and handle each case.
        match ty {
            SpandaType::Void => self.write("Void"),
            SpandaType::Int => self.write("Int"),
            SpandaType::Float => self.write("Float"),
            SpandaType::Bool => self.write("Bool"),
            SpandaType::String => self.write("String"),
            SpandaType::Char => self.write("Char"),
            SpandaType::Bytes => self.write("Bytes"),
            SpandaType::Null => self.write("Null"),
            SpandaType::Number { unit } => {
                self.write("Number");

                // Take the branch when *unit differs from None.
                if *unit != UnitKind::None {
                    self.space();
                    self.write(unit.as_str());
                }
            }
            SpandaType::Named { name } => self.write(name),
            SpandaType::Generic { name, type_args } => {
                self.write(name);
                self.write("<");

                // Iterate over enumerate with destructured elements.
                for (i, arg) in type_args.iter().enumerate() {
                    // Take this path when i > 0.
                    if i > 0 {
                        self.write(", ");
                    }
                    self.print_type(arg);
                }
                self.write(">");
            }
            SpandaType::TypeParam { name } => self.write(name),
            SpandaType::Scan => self.write("Scan"),
            SpandaType::Pose => self.write("Pose"),
            SpandaType::Velocity => self.write("Velocity"),
            SpandaType::Trajectory => self.write("Trajectory"),
            SpandaType::Transform => self.write("Transform"),
            SpandaType::EnumVariant { enum_name, variant } => {
                self.write(enum_name);
                self.write(".");
                self.write(variant);
            }
            SpandaType::TraitObject { trait_name } => {
                self.write("dyn ");
                self.write(trait_name);
            }
            SpandaType::Regex => self.write("Regex"),
            SpandaType::Match => self.write("Match"),
            SpandaType::Capture => self.write("Capture"),
            SpandaType::CaptureGroup => self.write("CaptureGroup"),
        }
    }

    fn visibility_prefix(v: Visibility) -> &'static str {
        // Visibility prefix.
        //
        // Parameters:
        // - `v` — input value
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::pretty::visibility_prefix(v);

        // Match on v and handle each case.
        match v {
            Visibility::Export => "export ",
            Visibility::Public => "public ",
            Visibility::Private => "private ",
        }
    }

    fn print_program(&mut self, source: &str, program: &Program) {
        // Print program.
        //
        // Parameters:
        // - `self` — method receiver
        // - `source` — input value
        // - `program` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_program(source, program);

        // Destructure the program into its top-level sections.
        let Program::Program {
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
            ..
        } = program;

        // Emit output when module name provides a name.
        if let Some(name) = module_name {
            self.write_line(&format!("module {name};"));

            // Skip further work when !imports is empty.
            if !imports.is_empty() || !functions.is_empty() || !tests.is_empty() {
                self.newline();
            }
        }

        // Iterate over enumerate with destructured elements.
        for (i, import) in imports.iter().enumerate() {
            let ImportDecl::ImportDecl { path, .. } = import;
            self.write_line(&format!("import {path};"));

            // Skip further work when len is empty.
            if i + 1 == imports.len() && (!functions.is_empty() || !tests.is_empty()) {
                self.newline();
            }
        }

        // Iterate over enumerate with destructured elements.
        for (i, func) in functions.iter().enumerate() {
            self.print_module_fn(func);

            // Take this path when i + 1 < functions.len().
            if i + 1 < functions.len() {
                self.newline();
            }
        }

        // Skip further work when !functions is empty.
        if !functions.is_empty() && (!tests.is_empty() || !structs.is_empty()) {
            self.newline();
        }

        // Iterate over enumerate with destructured elements.
        for (i, test) in tests.iter().enumerate() {
            self.print_test(test);

            // Take this path when i + 1 < tests.len().
            if i + 1 < tests.len() {
                self.newline();
            }
        }

        // Skip further work when !tests is empty.
        if !tests.is_empty() && !structs.is_empty() {
            self.newline();
        }

        // Iterate over enumerate with destructured elements.
        for (i, s) in structs.iter().enumerate() {
            self.print_struct(s);

            // Take this path when i + 1 < structs.len().
            if i + 1 < structs.len() {
                self.newline();
            }
        }

        // Skip further work when !structs is empty.
        if !structs.is_empty() && !enums.is_empty() {
            self.newline();
        }

        // Iterate over enumerate with destructured elements.
        for (i, e) in enums.iter().enumerate() {
            self.print_enum(e);

            // Take this path when i + 1 < enums.len().
            if i + 1 < enums.len() {
                self.newline();
            }
        }

        // Skip further work when !enums is empty.
        if !enums.is_empty() && !traits.is_empty() {
            self.newline();
        }

        // Iterate over enumerate with destructured elements.
        for (i, t) in traits.iter().enumerate() {
            self.print_trait(t);

            // Take this path when i + 1 < traits.len().
            if i + 1 < traits.len() {
                self.newline();
            }
        }

        // Process each hardware profile.
        for hw in hardware_profiles {
            self.emit_source_span(source, span_of(hw));
            self.newline();
        }

        // Emit output when requires hardware provides a req.
        if let Some(req) = requires_hardware {
            self.emit_source_span(source, span_of(req));
            self.newline();
        }

        // Emit output when requires network provides a req.
        if let Some(req) = requires_network {
            self.emit_source_span(source, span_of(req));
            self.newline();
        }

        // Emit output when simulate compatibility provides a sim.
        if let Some(sim) = simulate_compatibility {
            self.emit_source_span(source, span_of(sim));
            self.newline();
        }

        // Process each message.
        for msg in messages {
            self.emit_source_span(source, span_of(msg));
            self.newline();
        }

        // Resolve each dependency specification.
        for dep in deployments {
            self.emit_source_span(source, span_of(dep));
            self.newline();
        }

        // Iterate over enumerate with destructured elements.
        for (i, robot) in robots.iter().enumerate() {
            // Take this path when i > 0.
            if i > 0 {
                self.newline();
            }
            self.print_robot(source, robot);
        }
    }

    fn print_module_fn(&mut self, func: &crate::foundations::ModuleFnDecl) {
        // Print module fn.
        //
        // Parameters:
        // - `self` — method receiver
        // - `func` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_module_fn(func);

        // Create mutable header for accumulating results.
        let mut header = String::new();
        header.push_str(Self::visibility_prefix(func.visibility));

        // Skip synchronous handling for async functions.
        if func.is_async {
            header.push_str("async ");
        }
        header.push_str("fn ");
        header.push_str(&func.name);

        // Skip further work when type params is empty.
        if !func.type_params.is_empty() {
            header.push('<');
            header.push_str(&func.type_params.join(", "));
            header.push('>');
        }
        header.push('(');

        // Bind each formal parameter to its call argument.
        for (i, param) in func.params.iter().enumerate() {
            // Take this path when i > 0.
            if i > 0 {
                header.push_str(", ");
            }
            header.push_str(&param.name);
            header.push_str(": ");
            let mut ty = PrettyPrinter::new();
            ty.print_type(&param.type_ann);
            header.push_str(&ty.out);
        }
        header.push(')');
        header.push_str(" -> ");
        let mut ret = PrettyPrinter::new();
        ret.print_type(&func.return_type);
        header.push_str(&ret.out);
        self.open_block(&header);
        self.print_stmts(&func.body);
        self.close_block("");
    }

    fn print_test(&mut self, test: &crate::foundations::TestDecl) {
        // Print test.
        //
        // Parameters:
        // - `self` — method receiver
        // - `test` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_test(test);

        // Call open block on the current instance.
        self.open_block(&format!("test \"{}\"", test.name));
        self.print_stmts(&test.body);
        self.close_block("");
    }

    fn print_struct(&mut self, decl: &crate::foundations::StructDecl) {
        // Print struct.
        //
        // Parameters:
        // - `self` — method receiver
        // - `decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_struct(decl);

        // Compute crate for the following logic.
        let crate::foundations::StructDecl::StructDecl { name, fields, .. } = decl;
        self.open_block(&format!("struct {name}"));

        // Check each struct field.
        for field in fields {
            self.write_line(&format!("{}: {};", field.name, field.type_name));
        }
        self.close_block("");
    }

    fn print_enum(&mut self, decl: &crate::foundations::EnumDecl) {
        // Print enum.
        //
        // Parameters:
        // - `self` — method receiver
        // - `decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_enum(decl);

        // Compute crate for the following logic.
        let crate::foundations::EnumDecl::EnumDecl { name, variants, .. } = decl;
        self.open_block(&format!("enum {name}"));

        // Iterate over enumerate with destructured elements.
        for (i, variant) in variants.iter().enumerate() {
            let suffix = if i + 1 == variants.len() { "" } else { "," };

            // Skip further work when field types is empty.
            if variant.field_types.is_empty() {
                self.write_line(&format!("{}{suffix}", variant.name));
            } else {
                self.write_line(&format!(
                    "{}({}){suffix}",
                    variant.name,
                    variant.field_types.join(", ")
                ));
            }
        }
        self.close_block("");
    }

    fn print_trait(&mut self, decl: &crate::foundations::TraitDecl) {
        // Print trait.
        //
        // Parameters:
        // - `self` — method receiver
        // - `decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_trait(decl);

        // Compute crate for the following logic.
        let crate::foundations::TraitDecl::TraitDecl { name, methods, .. } = decl;
        self.open_block(&format!("trait {name}"));

        // Process each method.
        for method in methods {
            self.write_line(&format!(
                "fn {}({}) -> {};",
                method.name,
                method
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.type_name))
                    .collect::<Vec<_>>()
                    .join(", "),
                method.return_type
            ));
        }
        self.close_block("");
    }

    fn print_robot(&mut self, source: &str, robot: &RobotDecl) {
        // Print robot.
        //
        // Parameters:
        // - `self` — method receiver
        // - `source` — input value
        // - `robot` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_robot(source, robot);

        // Compute RobotDecl for the following logic.
        let RobotDecl::RobotDecl {
            name,
            sensors,
            actuators,
            safety,
            agents,
            behaviors,
            tasks,
            events,
            event_handlers,
            trait_impls,
            span,
            ..
        } = robot;

        // Skip further work when sensors is empty.
        if sensors.is_empty()
            && actuators.is_empty()
            && safety.is_none()
            && agents.is_empty()
            && behaviors.is_empty()
            && tasks.is_empty()
            && events.is_empty()
            && event_handlers.is_empty()
            && trait_impls.is_empty()
        {
            self.emit_source_span(source, span);
            return;
        }
        self.open_block(&format!("robot {name}"));

        // Process each sensor.
        for sensor in sensors {
            let SensorDecl::SensorDecl {
                name,
                sensor_type,
                binding,
                ..
            } = sensor;
            let mut line = format!("sensor {name}: {sensor_type}");

            // Take this path when let Some(SensorBinding::Topic { path }) = binding.
            if let Some(SensorBinding::Topic { path }) = binding {
                line.push_str(&format!(" on \"{path}\""));
            }
            line.push(';');
            self.write_line(&line);
        }

        // Process each actuator.
        for actuator in actuators {
            let ActuatorDecl::ActuatorDecl {
                name,
                actuator_type,
                ..
            } = actuator;
            self.write_line(&format!("actuator {name}: {actuator_type};"));
        }

        // Take this path when let Some(SafetyBlock::SafetyBlock { rules, .. }) = safety.
        if let Some(SafetyBlock::SafetyBlock { rules, .. }) = safety {
            self.open_block("safety");

            // Process each rule.
            for rule in rules {
                // Match on rule and handle each case.
                match rule {
                    SafetyRule::MaxSpeedRule {
                        name, value, unit, ..
                    } => {
                        let mut val = PrettyPrinter::new();
                        val.print_expr(value);
                        self.write(&format!("{name} = {}", val.out));

                        // Take the branch when *unit differs from None.
                        if *unit != UnitKind::None {
                            self.space();
                            self.write(unit.as_str());
                        }
                        self.write_line(";");
                    }
                    SafetyRule::StopIfRule { condition, .. } => {
                        let mut cond = PrettyPrinter::new();
                        cond.print_expr(condition);
                        self.write_line(&format!("stop_if {};", cond.out));
                    }
                }
            }
            self.close_block("");
        }

        // Process each agent.
        for agent in agents {
            let AgentDecl::AgentDecl {
                name,
                uses_ai,
                memory_kind,
                tools,
                skills,
                capabilities,
                goal,
                plan_body,
                ..
            } = agent;
            self.open_block(&format!("agent {name}"));

            // Iterate over uses ai.
            for model in uses_ai {
                self.write_line(&format!("uses {model};"));
            }

            // Skip further work when !tools is empty.
            if !tools.is_empty() {
                self.write_line(&format!("tools [{}];", tools.join(", ")));
            }

            // Emit output when memory kind provides a kind.
            if let Some(kind) = memory_kind {
                let mem = match kind {
                    MemoryKind::ShortTerm => "short_term",
                    MemoryKind::LongTerm => "long_term",
                };
                self.write_line(&format!("memory {mem};"));
            }

            // Process each skill.
            for skill in skills {
                self.write_line(&format!("skill {skill};"));
            }

            // Skip further work when !capabilities is empty.
            if !capabilities.is_empty() {
                let caps = capabilities
                    .iter()
                    .map(format_capability)
                    .collect::<Vec<_>>()
                    .join(", ");
                self.write_line(&format!("can [ {caps} ];"));
            }
            self.write_line(&format!("goal \"{goal}\";"));
            self.open_block("plan");
            self.print_stmts(plan_body);
            self.close_block("");
            self.close_block("");
        }

        // Process each event.
        for event in events {
            let crate::foundations::EventDecl::EventDecl { name, .. } = event;
            self.write_line(&format!("event {name};"));
        }

        // Invoke each registered handler.
        for handler in event_handlers {
            let crate::foundations::EventHandlerDecl::EventHandlerDecl {
                event_name, body, ..
            } = handler;
            self.open_block(&format!("on {event_name}"));
            self.print_stmts(body);
            self.close_block("");
        }

        // Process each behavior.
        for behavior in behaviors {
            let BehaviorDecl::BehaviorDecl {
                name,
                requires,
                ensures,
                body,
                ..
            } = behavior;
            let mut header = format!("behavior {name}()");

            // Emit output when requires provides a req.
            if let Some(req) = requires {
                let mut cond = PrettyPrinter::new();
                cond.print_expr(req);
                header.push_str(&format!(" requires {}", cond.out));
            }

            // Emit output when ensures provides a ens.
            if let Some(ens) = ensures {
                let mut cond = PrettyPrinter::new();
                cond.print_expr(ens);
                header.push_str(&format!(" ensures {}", cond.out));
            }
            self.open_block(&header);
            self.print_stmts(body);
            self.close_block("");
        }

        // Process each task.
        for task in tasks {
            let crate::foundations::TaskDecl::TaskDecl {
                name,
                priority,
                interval_ms,
                requires,
                ensures,
                body,
                ..
            } = task;
            let mut header = format!("task {name}");

            // Keep entries that match the expected pattern.
            if !matches!(priority, crate::foundations::TaskPriority::Normal) {
                header.push(' ');
                header.push_str(match priority {
                    crate::foundations::TaskPriority::Critical => "critical",
                    crate::foundations::TaskPriority::High => "high",
                    crate::foundations::TaskPriority::Normal => "normal",
                    crate::foundations::TaskPriority::Low => "low",
                });
            }
            header.push_str(&format!(" every {interval_ms}ms"));

            // Emit output when requires provides a req.
            if let Some(req) = requires {
                let mut cond = PrettyPrinter::new();
                cond.print_expr(req);
                header.push_str(&format!(" requires {}", cond.out));
            }

            // Emit output when ensures provides a ens.
            if let Some(ens) = ensures {
                let mut cond = PrettyPrinter::new();
                cond.print_expr(ens);
                header.push_str(&format!(" ensures {}", cond.out));
            }
            self.open_block(&header);
            self.print_stmts(body);
            self.close_block("");
        }

        // Process each trait impl.
        for imp in trait_impls {
            let crate::foundations::TraitImplDecl::TraitImplDecl {
                trait_name,
                agent_name,
                methods,
                ..
            } = imp;
            self.open_block(&format!("impl {trait_name} for {agent_name}"));

            // Process each method.
            for method in methods {
                let mut header = format!("fn {}(", method.name);
                header.push_str(
                    &method
                        .params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, p.type_name))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                header.push_str(&format!(") -> {}", method.return_type));
                self.open_block(&header);
                self.print_stmts(&method.body);
                self.close_block("");
            }
            self.close_block("");
        }
        self.close_block("");
    }

    fn print_stmts(&mut self, stmts: &[Stmt]) {
        // Print stmts.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmts` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_stmts(stmts);

        // Execute each statement in sequence.
        for stmt in stmts {
            self.print_stmt(stmt);
        }
    }

    fn print_stmt(&mut self, stmt: &Stmt) {
        // Print stmt.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmt` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_stmt(stmt);

        // Match on stmt and handle each case.
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation,
                init,
                ..
            } => {
                self.write(&format!("let {name}"));

                // Emit output when type annotation provides a ty.
                if let Some(ty) = type_annotation {
                    self.write(": ");
                    self.print_type(ty);
                }

                // Emit output when init provides a value.
                if let Some(value) = init {
                    self.write(" = ");
                    self.print_expr(value);
                }
                self.write_line(";");
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.write("if ");
                self.print_expr(condition);
                self.write_line(" {");
                self.indent += 1;
                self.print_stmts(then_branch);
                self.indent = self.indent.saturating_sub(1);

                // Emit output when else branch provides a else body.
                if let Some(else_body) = else_branch {
                    self.write_line("} else {");
                    self.indent += 1;
                    self.print_stmts(else_body);
                    self.indent = self.indent.saturating_sub(1);
                    self.write_line("}");
                } else {
                    self.write_line("}");
                }
            }
            Stmt::LoopStmt {
                interval_ms, body, ..
            } => {
                self.open_block(&format!("loop every {interval_ms}ms"));
                self.print_stmts(body);
                self.close_block("");
            }
            Stmt::ReturnStmt { value, .. } => {
                self.write("return");

                // Emit output when value provides a v.
                if let Some(v) = value {
                    self.space();
                    self.print_expr(v);
                }
                self.write_line(";");
            }
            Stmt::ExprStmt { expr, .. } => {
                self.print_expr(expr);
                self.write_line(";");
            }
            Stmt::PublishStmt {
                topic_name, value, ..
            } => {
                self.write(&format!("publish({topic_name}, "));
                self.print_expr(value);
                self.write_line(");");
            }
            Stmt::ServiceCallStmt { service_name, .. } => {
                self.write_line(&format!("call {service_name}();"));
            }
            Stmt::ActionSendStmt {
                action_name, goal, ..
            } => {
                self.write(&format!("send_goal {action_name} with "));
                self.print_expr(goal);
                self.write_line(";");
            }
            Stmt::EmergencyStopStmt { .. } => self.write_line("emergency_stop;"),
            Stmt::ResetEmergencyStopStmt { .. } => self.write_line("reset_emergency_stop;"),
            Stmt::EmitStmt { event_name, .. } => {
                self.write_line(&format!("emit {event_name};"));
            }
            Stmt::EnterStmt { state_name, .. } => {
                self.write_line(&format!("enter {state_name};"));
            }
            Stmt::RememberStmt { key, value, .. } => {
                self.write(&format!("remember(\"{key}\", "));
                self.print_expr(value);
                self.write_line(");");
            }
            Stmt::SubscribeStmt { target, .. } => {
                self.write_line(&format!("subscribe({target});"));
            }
            Stmt::ExecuteStmt {
                action_name, goal, ..
            } => {
                self.write(&format!("execute({action_name}, "));
                self.print_expr(goal);
                self.write_line(");");
            }
            Stmt::DiscoverStmt { target, filter, .. } => {
                self.write("discover(");
                self.write(format_discover_target(*target));

                // Emit output when filter provides a f.
                if let Some(f) = filter {
                    self.write(" ");
                    self.write(&format_discover_filter(f));
                }
                self.write_line(");");
            }
            Stmt::ReceiveStmt {
                topic_name,
                var_name,
                ..
            } => {
                self.write_line(&format!("receive {topic_name} into {var_name};"));
            }
            Stmt::SpawnStmt { callee, args, .. } => {
                self.write("spawn ");
                self.print_expr(callee);

                // Skip further work when !args is empty.
                if !args.is_empty() {
                    self.write("(");

                    // Iterate over enumerate with destructured elements.
                    for (i, arg) in args.iter().enumerate() {
                        // Take this path when i > 0.
                        if i > 0 {
                            self.write(", ");
                        }
                        self.print_expr(arg);
                    }
                    self.write(")");
                }
                self.write_line(";");
            }
            Stmt::SelectStmt { arms, .. } => {
                self.open_block("select");

                // Process each arm.
                for arm in arms {
                    self.write("recv(");
                    self.print_expr(&arm.channel);
                    self.write(") => ");

                    // Take the branch when len equals 1.
                    if arm.body.len() == 1 {
                        self.print_stmt(&arm.body[0]);
                    } else {
                        self.open_block("");
                        self.print_stmts(&arm.body);
                        self.close_block("");
                    }
                }
                self.close_block(";");
            }
            Stmt::ParallelStmt { body, .. } => {
                self.open_block("parallel");
                self.print_stmts(body);
                self.close_block(";");
            }
            Stmt::EnterModeStmt { mode, .. } => {
                self.write_line(&format!("enter {mode}_mode;"));
            }
            Stmt::UseFallbackStmt { resource, .. } => {
                self.write_line(&format!("use {resource};"));
            }
            Stmt::StopAllActuatorsStmt { .. } => {
                self.write_line("stop_all_actuators();");
            }
            Stmt::RunPipelineStmt { name, .. } => {
                self.write_line(&format!("run_pipeline {name};"));
            }
        }
    }

    fn print_expr(&mut self, expr: &Expr) {
        // Print expr.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expr` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.print_expr(expr);

        // Match on expr and handle each case.
        match expr {
            Expr::LiteralExpr { value, .. } => match value {
                LiteralValue::Number(n) => self.write(&format_number(*n)),
                LiteralValue::String(s) => self.write(&format!("\"{}\"", escape_string(s))),
                LiteralValue::Bool(b) => self.write(if *b { "true" } else { "false" }),
                LiteralValue::Null => self.write("null"),
                LiteralValue::Regex(pattern) => {
                    self.write(&format!("/{}/{}", pattern.source, pattern.flags));
                }
            },
            Expr::UnitLiteralExpr { value, unit, .. } => {
                self.write(&format_number(*value));

                // Take the branch when *unit differs from None.
                if *unit != UnitKind::None {
                    self.space();
                    self.write(unit.as_str());
                }
            }
            Expr::IdentExpr { name, .. } => self.write(name),
            Expr::BinaryExpr {
                op, left, right, ..
            } => {
                self.print_expr(left);
                self.space();
                self.write(op.as_str());
                self.space();
                self.print_expr(right);
            }
            Expr::UnaryExpr { op, operand, .. } => {
                // Match on op and handle each case.
                match op {
                    UnaryOp::Neg => self.write("-"),
                    UnaryOp::Not => self.write("not "),
                }
                self.print_expr(operand);
            }
            Expr::CallExpr {
                callee,
                args,
                named_args,
                ..
            } => {
                self.print_expr(callee);
                self.write("(");
                let mut first = true;

                // Apply each command-line argument.
                for arg in args {
                    // Take the branch when first is false.
                    if !first {
                        self.write(", ");
                    }
                    first = false;
                    self.print_expr(arg);
                }

                // Process each named arg.
                for named in named_args {
                    // Take the branch when first is false.
                    if !first {
                        self.write(", ");
                    }
                    first = false;
                    self.write(&format!("{}: ", named.name));
                    self.print_expr(&named.value);
                }
                self.write(")");
            }
            Expr::MemberExpr {
                object, property, ..
            } => {
                self.print_expr(object);
                self.write(".");
                self.write(property);
            }
            Expr::MatchExpr {
                scrutinee, arms, ..
            } => {
                self.write("match ");
                self.print_expr(scrutinee);
                self.write_line(" {");
                self.indent += 1;

                // Iterate over enumerate with destructured elements.
                for (i, arm) in arms.iter().enumerate() {
                    self.write(&arm.variant);
                    self.write(" => ");

                    // Take the branch when len equals 1.
                    if arm.body.len() == 1 {
                        self.print_stmt(&arm.body[0]);
                    } else {
                        self.open_block("");
                        self.print_stmts(&arm.body);
                        self.close_block("");
                    }

                    // Take this path when i + 1 < arms.len().
                    if i + 1 < arms.len() {
                        self.newline();
                    }
                }
                self.indent = self.indent.saturating_sub(1);
                self.write_line("};");
            }
            Expr::StructLiteralExpr {
                type_name, fields, ..
            } => {
                self.write(type_name);
                self.write(" { ");

                // Iterate over enumerate with destructured elements.
                for (i, field) in fields.iter().enumerate() {
                    // Take this path when i > 0.
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&format!("{}: ", field.name));
                    self.print_expr(&field.value);
                }
                self.write(" }");
            }
            Expr::ServiceCallExpr { service_name, .. } => {
                self.write(&format!("call {service_name}()"));
            }
            Expr::ExecuteExpr {
                action_name, goal, ..
            } => {
                self.write(&format!("execute({action_name}, "));
                self.print_expr(goal);
                self.write(")");
            }
            Expr::DiscoverExpr { target, filter, .. } => {
                self.write("discover(");
                self.write(format_discover_target(*target));

                // Emit output when filter provides a f.
                if let Some(f) = filter {
                    self.write(" ");
                    self.write(&format_discover_filter(f));
                }
                self.write(")");
            }
            Expr::AwaitExpr { operand, .. } => {
                self.write("await ");
                self.print_expr(operand);
            }
            Expr::SpawnExpr { callee, args, .. } => {
                self.write("spawn ");

                // Take this path when let Expr::IdentExpr { name, .. } = callee.as ref().
                if let Expr::IdentExpr { name, .. } = callee.as_ref() {
                    self.write(name);
                }

                // Skip further work when !args is empty.
                if !args.is_empty() {
                    self.write("(");

                    // Iterate over enumerate with destructured elements.
                    for (i, arg) in args.iter().enumerate() {
                        // Take this path when i > 0.
                        if i > 0 {
                            self.write(", ");
                        }
                        self.print_expr(arg);
                    }
                    self.write(")");
                }
            }
        }
    }
}

fn span_of<T: HasSpan>(value: &T) -> &Span {
    // Span of.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // &Span.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::pretty::span_of(value);

    // Produce span as the result.
    value.span()
}

trait HasSpan {
    fn span(&self) -> &Span;
}

impl HasSpan for crate::foundations::HardwareDecl {
    fn span(&self) -> &Span {
        // Span.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &Span.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.span();

        // Dispatch based on the enum variant or current state.
        match self {
            crate::foundations::HardwareDecl::HardwareDecl { span, .. } => span,
        }
    }
}

impl HasSpan for crate::foundations::DeployDecl {
    fn span(&self) -> &Span {
        // Span.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &Span.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.span();

        // Dispatch based on the enum variant or current state.
        match self {
            crate::foundations::DeployDecl::DeployDecl { span, .. } => span,
        }
    }
}

impl HasSpan for crate::foundations::RequiresHardwareDecl {
    fn span(&self) -> &Span {
        // Span.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &Span.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.span();

        // Dispatch based on the enum variant or current state.
        match self {
            crate::foundations::RequiresHardwareDecl::RequiresHardwareDecl { span, .. } => span,
        }
    }
}

impl HasSpan for crate::foundations::RequiresNetworkDecl {
    fn span(&self) -> &Span {
        // Span.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &Span.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.span();

        // Dispatch based on the enum variant or current state.
        match self {
            crate::foundations::RequiresNetworkDecl::RequiresNetworkDecl { span, .. } => span,
        }
    }
}

impl HasSpan for crate::foundations::SimulateCompatibilityDecl {
    fn span(&self) -> &Span {
        // Span.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &Span.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.span();

        // Dispatch based on the enum variant or current state.
        match self {
            crate::foundations::SimulateCompatibilityDecl::SimulateCompatibilityDecl {
                span,
                ..
            } => span,
        }
    }
}

impl HasSpan for crate::comm::MessageDecl {
    fn span(&self) -> &Span {
        // Span.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &Span.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.span();

        // Dispatch based on the enum variant or current state.
        match self {
            crate::comm::MessageDecl::MessageDecl { span, .. } => span,
        }
    }
}

fn format_capability(cap: &CapabilityDecl) -> String {
    // Format capability.
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::pretty::format_capability(cap);

    // use target when target is present.

    // Emit output when target provides a target.
    if let Some(target) = &cap.target {
        format!("{}({target})", cap.action)
    } else {
        cap.action.clone()
    }
}

fn format_discover_target(target: DiscoverTarget) -> &'static str {
    // Format discover target.
    //
    // Parameters:
    // - `target` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::pretty::format_discover_target(target);

    // Match on target and handle each case.
    match target {
        DiscoverTarget::Robots => "robots",
        DiscoverTarget::Agents => "agents",
        DiscoverTarget::Devices => "devices",
    }
}

fn format_discover_filter(filter: &DiscoverFilter) -> String {
    // Format discover filter.
    //
    // Parameters:
    // - `filter` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::pretty::format_discover_filter(filter);

    // use cap when capability is present.

    // Emit output when capability provides a cap.
    if let Some(cap) = &filter.capability {
        format!("where capability includes {cap}")
    } else {
        String::new()
    }
}

fn format_number(n: f64) -> String {
    // Format number.
    //
    // Parameters:
    // - `n` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::pretty::format_number(n);

    // take this path when (n - n.round()).abs() < f64::EPSILON.
    if (n - n.round()).abs() < f64::EPSILON {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

fn escape_string(s: &str) -> String {
    // Escape string.
    //
    // Parameters:
    // - `s` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::pretty::escape_string(s);

    // Produce replace as the result.
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn pretty_print_program(source: &str, program: &Program) -> String {
    // Pretty print program.
    //
    // Parameters:
    // - `source` — input value
    // - `program` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::pretty::pretty_print_program(source, program);

    // Create mutable printer for accumulating results.
    let mut printer = PrettyPrinter::new();
    printer.print_program(source, program);
    printer.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{format_ast, lexer, parser};

    #[test]
    fn pretty_print_normalizes_module_fn() {
        // Pretty print normalizes module fn.
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
        // let result = spanda_core::pretty::pretty_print_normalizes_module_fn();

        let source = r#"module math;
export fn add(x:Int,y:Int)->Int{return x;}
"#;
        let tokens = lexer::tokenize(source).unwrap();
        let program = parser::parse(tokens).unwrap();
        let formatted = pretty_print_program(source, &program);
        assert!(formatted.contains("export fn add(x: Int, y: Int) -> Int"));
        assert!(formatted.contains("return x;"));
    }

    #[test]
    fn format_ast_round_trip_parseable() {
        // Format ast round trip parseable.
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
        // let result = spanda_core::pretty::format_ast_round_trip_parseable();

        let source = r#"
module demo;

export fn ping() -> Int {
  return 1;
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let x = ping();
    wheels.stop();
  }
}
"#;
        let formatted = format_ast(source).unwrap();
        let tokens = lexer::tokenize(&formatted).unwrap();
        assert!(parser::parse(tokens).is_ok());
    }
}
