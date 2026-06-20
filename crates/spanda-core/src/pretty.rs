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
        Self {
            out: String::new(),
            indent: 0,
            at_line_start: true,
        }
    }

    fn finish(mut self) -> String {
        while self.out.ends_with('\n') {
            self.out.pop();
        }
        self.out.push('\n');
        self.out
    }

    fn write_indent(&mut self) {
        if self.at_line_start {
            for _ in 0..self.indent {
                self.out.push_str("  ");
            }
            self.at_line_start = false;
        }
    }

    fn write(&mut self, text: &str) {
        self.write_indent();
        self.out.push_str(text);
    }

    fn space(&mut self) {
        if !self.at_line_start && !self.out.ends_with(' ') && !self.out.ends_with('\n') {
            self.out.push(' ');
        }
    }

    fn newline(&mut self) {
        self.out.push('\n');
        self.at_line_start = true;
    }

    fn write_line(&mut self, text: &str) {
        self.write(text);
        self.newline();
    }

    fn open_block(&mut self, header: &str) {
        self.write_line(&format!("{header} {{"));
        self.indent += 1;
    }

    fn close_block(&mut self, suffix: &str) {
        self.indent = self.indent.saturating_sub(1);
        self.write_line(&format!("}}{suffix}"));
    }

    fn emit_source_span(&mut self, source: &str, span: &Span) {
        let Some(chunk) = source.get(span.start.offset..span.end.offset) else {
            return;
        };
        for line in chunk.lines() {
            self.write_line(line.trim_end());
        }
    }

    fn print_type(&mut self, ty: &SpandaType) {
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
                if *unit != UnitKind::None {
                    self.space();
                    self.write(unit.as_str());
                }
            }
            SpandaType::Named { name } => self.write(name),
            SpandaType::Generic { name, type_args } => {
                self.write(name);
                self.write("<");
                for (i, arg) in type_args.iter().enumerate() {
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
        }
    }

    fn visibility_prefix(v: Visibility) -> &'static str {
        match v {
            Visibility::Export => "export ",
            Visibility::Public => "public ",
            Visibility::Private => "private ",
        }
    }

    fn print_program(&mut self, source: &str, program: &Program) {
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

        if let Some(name) = module_name {
            self.write_line(&format!("module {name};"));
            if !imports.is_empty() || !functions.is_empty() || !tests.is_empty() {
                self.newline();
            }
        }

        for (i, import) in imports.iter().enumerate() {
            let ImportDecl::ImportDecl { path, .. } = import;
            self.write_line(&format!("import {path};"));
            if i + 1 == imports.len() && (!functions.is_empty() || !tests.is_empty()) {
                self.newline();
            }
        }

        for (i, func) in functions.iter().enumerate() {
            self.print_module_fn(func);
            if i + 1 < functions.len() {
                self.newline();
            }
        }
        if !functions.is_empty() && (!tests.is_empty() || !structs.is_empty()) {
            self.newline();
        }

        for (i, test) in tests.iter().enumerate() {
            self.print_test(test);
            if i + 1 < tests.len() {
                self.newline();
            }
        }
        if !tests.is_empty() && !structs.is_empty() {
            self.newline();
        }

        for (i, s) in structs.iter().enumerate() {
            self.print_struct(s);
            if i + 1 < structs.len() {
                self.newline();
            }
        }
        if !structs.is_empty() && !enums.is_empty() {
            self.newline();
        }

        for (i, e) in enums.iter().enumerate() {
            self.print_enum(e);
            if i + 1 < enums.len() {
                self.newline();
            }
        }
        if !enums.is_empty() && !traits.is_empty() {
            self.newline();
        }

        for (i, t) in traits.iter().enumerate() {
            self.print_trait(t);
            if i + 1 < traits.len() {
                self.newline();
            }
        }

        for hw in hardware_profiles {
            self.emit_source_span(source, span_of(hw));
            self.newline();
        }
        if let Some(req) = requires_hardware {
            self.emit_source_span(source, span_of(req));
            self.newline();
        }
        if let Some(req) = requires_network {
            self.emit_source_span(source, span_of(req));
            self.newline();
        }
        if let Some(sim) = simulate_compatibility {
            self.emit_source_span(source, span_of(sim));
            self.newline();
        }
        for msg in messages {
            self.emit_source_span(source, span_of(msg));
            self.newline();
        }
        for dep in deployments {
            self.emit_source_span(source, span_of(dep));
            self.newline();
        }

        for (i, robot) in robots.iter().enumerate() {
            if i > 0 {
                self.newline();
            }
            self.print_robot(source, robot);
        }
    }

    fn print_module_fn(&mut self, func: &crate::foundations::ModuleFnDecl) {
        let mut header = String::new();
        header.push_str(Self::visibility_prefix(func.visibility));
        if func.is_async {
            header.push_str("async ");
        }
        header.push_str("fn ");
        header.push_str(&func.name);
        if !func.type_params.is_empty() {
            header.push('<');
            header.push_str(&func.type_params.join(", "));
            header.push('>');
        }
        header.push('(');
        for (i, param) in func.params.iter().enumerate() {
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
        self.open_block(&format!("test \"{}\"", test.name));
        self.print_stmts(&test.body);
        self.close_block("");
    }

    fn print_struct(&mut self, decl: &crate::foundations::StructDecl) {
        let crate::foundations::StructDecl::StructDecl { name, fields, .. } = decl;
        self.open_block(&format!("struct {name}"));
        for field in fields {
            self.write_line(&format!("{}: {};", field.name, field.type_name));
        }
        self.close_block("");
    }

    fn print_enum(&mut self, decl: &crate::foundations::EnumDecl) {
        let crate::foundations::EnumDecl::EnumDecl { name, variants, .. } = decl;
        self.open_block(&format!("enum {name}"));
        for (i, variant) in variants.iter().enumerate() {
            let suffix = if i + 1 == variants.len() { "" } else { "," };
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
        let crate::foundations::TraitDecl::TraitDecl { name, methods, .. } = decl;
        self.open_block(&format!("trait {name}"));
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
        for sensor in sensors {
            let SensorDecl::SensorDecl {
                name,
                sensor_type,
                binding,
                ..
            } = sensor;
            let mut line = format!("sensor {name}: {sensor_type}");
            if let Some(SensorBinding::Topic { path }) = binding {
                line.push_str(&format!(" on \"{path}\""));
            }
            line.push(';');
            self.write_line(&line);
        }
        for actuator in actuators {
            let ActuatorDecl::ActuatorDecl {
                name,
                actuator_type,
                ..
            } = actuator;
            self.write_line(&format!("actuator {name}: {actuator_type};"));
        }
        if let Some(SafetyBlock::SafetyBlock { rules, .. }) = safety {
            self.open_block("safety");
            for rule in rules {
                match rule {
                    SafetyRule::MaxSpeedRule {
                        name, value, unit, ..
                    } => {
                        let mut val = PrettyPrinter::new();
                        val.print_expr(value);
                        self.write(&format!("{name} = {}", val.out));
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
            for model in uses_ai {
                self.write_line(&format!("uses {model};"));
            }
            if !tools.is_empty() {
                self.write_line(&format!("tools [{}];", tools.join(", ")));
            }
            if let Some(kind) = memory_kind {
                let mem = match kind {
                    MemoryKind::ShortTerm => "short_term",
                    MemoryKind::LongTerm => "long_term",
                };
                self.write_line(&format!("memory {mem};"));
            }
            for skill in skills {
                self.write_line(&format!("skill {skill};"));
            }
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
        for event in events {
            let crate::foundations::EventDecl::EventDecl { name, .. } = event;
            self.write_line(&format!("event {name};"));
        }
        for handler in event_handlers {
            let crate::foundations::EventHandlerDecl::EventHandlerDecl {
                event_name, body, ..
            } = handler;
            self.open_block(&format!("on {event_name}"));
            self.print_stmts(body);
            self.close_block("");
        }
        for behavior in behaviors {
            let BehaviorDecl::BehaviorDecl {
                name,
                requires,
                ensures,
                body,
                ..
            } = behavior;
            let mut header = format!("behavior {name}()");
            if let Some(req) = requires {
                let mut cond = PrettyPrinter::new();
                cond.print_expr(req);
                header.push_str(&format!(" requires {}", cond.out));
            }
            if let Some(ens) = ensures {
                let mut cond = PrettyPrinter::new();
                cond.print_expr(ens);
                header.push_str(&format!(" ensures {}", cond.out));
            }
            self.open_block(&header);
            self.print_stmts(body);
            self.close_block("");
        }
        for task in tasks {
            let crate::foundations::TaskDecl::TaskDecl {
                name,
                interval_ms,
                requires,
                ensures,
                body,
                ..
            } = task;
            let mut header = format!("task {name} every {interval_ms}ms");
            if let Some(req) = requires {
                let mut cond = PrettyPrinter::new();
                cond.print_expr(req);
                header.push_str(&format!(" requires {}", cond.out));
            }
            if let Some(ens) = ensures {
                let mut cond = PrettyPrinter::new();
                cond.print_expr(ens);
                header.push_str(&format!(" ensures {}", cond.out));
            }
            self.open_block(&header);
            self.print_stmts(body);
            self.close_block("");
        }
        for imp in trait_impls {
            let crate::foundations::TraitImplDecl::TraitImplDecl {
                trait_name,
                agent_name,
                methods,
                ..
            } = imp;
            self.open_block(&format!("impl {trait_name} for {agent_name}"));
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
        for stmt in stmts {
            self.print_stmt(stmt);
        }
    }

    fn print_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation,
                init,
                ..
            } => {
                self.write(&format!("let {name}"));
                if let Some(ty) = type_annotation {
                    self.write(": ");
                    self.print_type(ty);
                }
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
                if !args.is_empty() {
                    self.write("(");
                    for (i, arg) in args.iter().enumerate() {
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
                for arm in arms {
                    self.write("recv(");
                    self.print_expr(&arm.channel);
                    self.write(") => ");
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
        }
    }

    fn print_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::LiteralExpr { value, .. } => match value {
                LiteralValue::Number(n) => self.write(&format_number(*n)),
                LiteralValue::String(s) => self.write(&format!("\"{}\"", escape_string(s))),
                LiteralValue::Bool(b) => self.write(if *b { "true" } else { "false" }),
                LiteralValue::Null => self.write("null"),
            },
            Expr::UnitLiteralExpr { value, unit, .. } => {
                self.write(&format_number(*value));
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
                for arg in args {
                    if !first {
                        self.write(", ");
                    }
                    first = false;
                    self.print_expr(arg);
                }
                for named in named_args {
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
                for (i, arm) in arms.iter().enumerate() {
                    self.write(&arm.variant);
                    self.write(" => ");
                    if arm.body.len() == 1 {
                        self.print_stmt(&arm.body[0]);
                    } else {
                        self.open_block("");
                        self.print_stmts(&arm.body);
                        self.close_block("");
                    }
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
                for (i, field) in fields.iter().enumerate() {
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
        }
    }
}

fn span_of<T: HasSpan>(value: &T) -> &Span {
    value.span()
}

trait HasSpan {
    fn span(&self) -> &Span;
}

impl HasSpan for crate::foundations::HardwareDecl {
    fn span(&self) -> &Span {
        match self {
            crate::foundations::HardwareDecl::HardwareDecl { span, .. } => span,
        }
    }
}

impl HasSpan for crate::foundations::DeployDecl {
    fn span(&self) -> &Span {
        match self {
            crate::foundations::DeployDecl::DeployDecl { span, .. } => span,
        }
    }
}

impl HasSpan for crate::foundations::RequiresHardwareDecl {
    fn span(&self) -> &Span {
        match self {
            crate::foundations::RequiresHardwareDecl::RequiresHardwareDecl { span, .. } => span,
        }
    }
}

impl HasSpan for crate::foundations::RequiresNetworkDecl {
    fn span(&self) -> &Span {
        match self {
            crate::foundations::RequiresNetworkDecl::RequiresNetworkDecl { span, .. } => span,
        }
    }
}

impl HasSpan for crate::foundations::SimulateCompatibilityDecl {
    fn span(&self) -> &Span {
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
        match self {
            crate::comm::MessageDecl::MessageDecl { span, .. } => span,
        }
    }
}

fn format_capability(cap: &CapabilityDecl) -> String {
    if let Some(target) = &cap.target {
        format!("{}({target})", cap.action)
    } else {
        cap.action.clone()
    }
}

fn format_discover_target(target: DiscoverTarget) -> &'static str {
    match target {
        DiscoverTarget::Robots => "robots",
        DiscoverTarget::Agents => "agents",
        DiscoverTarget::Devices => "devices",
    }
}

fn format_discover_filter(filter: &DiscoverFilter) -> String {
    if let Some(cap) = &filter.capability {
        format!("where capability includes {cap}")
    } else {
        String::new()
    }
}

fn format_number(n: f64) -> String {
    if (n - n.round()).abs() < f64::EPSILON {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn pretty_print_program(source: &str, program: &Program) -> String {
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
