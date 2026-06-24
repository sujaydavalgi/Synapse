/**
 * extension module (extension.ts).
 * @module
 */

import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import {
  type LanguageClientOptions,
  LanguageClient,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

const BUILTIN_PROFILES = ["RoverV1", "RoverV2", "JetsonOrin", "RaspberryPi5", "ESP32"];

function bundledServerModule(context: vscode.ExtensionContext): string {
  // Description:
  //     BundledServerModule.
  //
  // Inputs:
  //     context: vscode.ExtensionContext
  //         Caller-supplied context.
  //
  // Outputs:
  //     result: string
  //         Return value from `bundledServerModule`.
  //
  // Example:

  //     const result = bundledServerModule(context);

  return path.join(context.extensionPath, "server", "dist", "server.js");
}

function resolveServerModule(context: vscode.ExtensionContext): string | null {
  // Description:
  //     ResolveServerModule.
  //
  // Inputs:
  //     context: vscode.ExtensionContext
  //         Caller-supplied context.
  //
  // Outputs:
  //     result: string | null
  //         Return value from `resolveServerModule`.
  //
  // Example:
  //     const result = resolveServerModule(context);
  // Description:
  //     ResolveServerModule.
  //
  // Inputs:
  //     context: vscode.ExtensionContext
  //         Caller-supplied context.
  //
  // Outputs:
  //     result: string | null
  //         Return value from `resolveServerModule`.
  //
  // Example:

  //     const result = resolveServerModule(context);

  const bundled = bundledServerModule(context);
  if (fs.existsSync(bundled)) {
    return bundled;
  }

  const cfg = vscode.workspace.getConfiguration("spanda");
  const configured = cfg.get<string>("languageServerPath");
  if (configured?.trim()) {
    return configured;
  }

  const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (workspace) {
    const workspaceServer = path.join(workspace, "packages/lsp/dist/server.js");
    if (fs.existsSync(workspaceServer)) {
      return workspaceServer;
    }
  }

  return null;
}

function resolveCliPath(): string | undefined {
  // Description:
  //     ResolveCliPath.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string | undefined
  //         Return value from `resolveCliPath`.
  //
  // Example:
  //     const result = resolveCliPath();
  // Description:
  //     ResolveCliPath.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string | undefined
  //         Return value from `resolveCliPath`.
  //
  // Example:

  //     const result = resolveCliPath();

  const cfg = vscode.workspace.getConfiguration("spanda");
  const configured = cfg.get<string>("cliPath");
  if (configured?.trim() && fs.existsSync(configured)) {
    return configured;
  }

  const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (workspace) {
    for (const candidate of [
      path.join(workspace, "target/release/spanda"),
      path.join(workspace, "target/debug/spanda"),
    ]) {
      if (fs.existsSync(candidate)) {
        return candidate;
      }
    }
  }

  const which = spawnSync("which", ["spanda"], { encoding: "utf-8" });
  if (which.status === 0 && which.stdout.trim()) {
    return which.stdout.trim();
  }
  return undefined;
}

function resolveDapPath(cliPath?: string): string | undefined {
  // Description:
  //     ResolveDapPath.
  //
  // Inputs:
  //     cliPath?: string
  //         Caller-supplied cliPath?.
  //
  // Outputs:
  //     result: string | undefined
  //         Return value from `resolveDapPath`.
  //
  // Example:
  //     const result = resolveDapPath(cliPath?);
  // Description:
  //     ResolveDapPath.
  //
  // Inputs:
  //     cliPath?: string
  //         Caller-supplied cliPath?.
  //
  // Outputs:
  //     result: string | undefined
  //         Return value from `resolveDapPath`.
  //
  // Example:

  //     const result = resolveDapPath(cliPath?);

  if (cliPath) {
    const sibling = path.join(path.dirname(cliPath), "spanda-dap");
    if (fs.existsSync(sibling)) {
      return sibling;
    }
  }

  const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (workspace) {
    for (const candidate of [
      path.join(workspace, "target/release/spanda-dap"),
      path.join(workspace, "target/debug/spanda-dap"),
    ]) {
      if (fs.existsSync(candidate)) {
        return candidate;
      }
    }
  }

  const which = spawnSync("which", ["spanda-dap"], { encoding: "utf-8" });
  if (which.status === 0 && which.stdout.trim()) {
    return which.stdout.trim();
  }
  return undefined;
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  // Description:
  //     Activate.
  //
  // Inputs:
  //     context: vscode.ExtensionContext
  //         Caller-supplied context.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `activate`.
  //
  // Example:
  //     const result = activate(context);
  // Description:
  //     Activate.
  //
  // Inputs:
  //     context: vscode.ExtensionContext
  //         Caller-supplied context.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `activate`.
  //
  // Example:

  //     const result = activate(context);

  const serverModule = resolveServerModule(context);
  if (!serverModule) {
    vscode.window.showWarningMessage("Spanda: language server path could not be resolved.");
    return;
  }

  const cliPath = resolveCliPath();
  const serverEnv: NodeJS.ProcessEnv = {
    ...process.env,
    SPANDA_EXTENSION_ROOT: context.extensionPath,
  };
  if (cliPath) {
    serverEnv.SPANDA_CLI_PATH = cliPath;
  }

  const run: ServerOptions = {
    module: serverModule,
    transport: TransportKind.ipc,
    options: { env: serverEnv },
  };

  const debug: ServerOptions = {
    module: serverModule,
    transport: TransportKind.ipc,
    options: {
      env: serverEnv,
      execArgv: ["--nolazy", "--inspect=6011"],
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "spanda" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.sd"),
    },
  };

  client = new LanguageClient("spanda-lsp", "Spanda Language Server", { run, debug }, clientOptions);
  context.subscriptions.push(client);
  await client.start();

  if (!cliPath) {
    void vscode.window.showInformationMessage(
      "Spanda: install the CLI for verify diagnostics — see editor/vscode/README.md",
    );
  }

  context.subscriptions.push(
    vscode.commands.registerCommand("spanda.verifyWithTarget", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "spanda") {
        void vscode.window.showWarningMessage("Open a .sd file to verify.");
        return;
      }
      const pick = await vscode.window.showQuickPick(BUILTIN_PROFILES, {
        placeHolder: "Select hardware deploy target",
        title: "spanda verify",
      });
      if (!pick) {
        return;
      }
      if (!cliPath) {
        void vscode.window.showErrorMessage("Spanda CLI not found. Set spanda.cliPath in settings.");
        return;
      }
      const file = editor.document.uri.fsPath;
      const result = spawnSync(cliPath, ["verify", file, "--target", pick], { encoding: "utf-8" });
      const channel = vscode.window.createOutputChannel("Spanda Verify");
      channel.clear();
      channel.appendLine(result.stdout || result.stderr);
      channel.show(true);
      if (result.status !== 0) {
        void vscode.window.showErrorMessage(`Deploy incompatible with ${pick}`);
      } else {
        void vscode.window.showInformationMessage(`✓ Compatible with ${pick}`);
      }
    }),
  );

  context.subscriptions.push(
    vscode.debug.registerDebugConfigurationProvider("spanda", {
      resolveDebugConfiguration(
        _folder: vscode.WorkspaceFolder | undefined,
        config: vscode.DebugConfiguration,
      ) {
        if (!config.program && vscode.window.activeTextEditor?.document.languageId === "spanda") {
          config.program = vscode.window.activeTextEditor.document.uri.fsPath;
        }
        if (!config.type) {
          config.type = "spanda";
        }
        if (!config.request) {
          config.request = "launch";
        }
        if (!config.name) {
          config.name = "Spanda Debug";
        }
        return config;
      },
    }),
  );

  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory("spanda", {
      createDebugAdapterDescriptor(session: vscode.DebugSession) {
        const dap = resolveDapPath(cliPath);
        const program = session.configuration.program as string | undefined;
        if (!dap) {
          void vscode.window.showErrorMessage(
            "spanda-dap not found. Build with `cargo build -p spanda-dap --release`.",
          );
          return new vscode.DebugAdapterExecutable("spanda-dap", program ? [program] : []);
        }
        return new vscode.DebugAdapterExecutable(dap, program ? [program] : []);
      },
    }),
  );
}

export async function deactivate(): Promise<void> {
  // Description:
  //     Deactivate.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `deactivate`.
  //
  // Example:
  //     const result = deactivate();
  // Description:
  //     Deactivate.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `deactivate`.
  //
  // Example:

  //     const result = deactivate();

  if (client) {
    await client.stop();
  }
}
