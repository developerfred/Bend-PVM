// The module 'vscode' contains the VS Code extensibility API
// Import the necessary extentions
import {
  workspace,
  window,
  ExtensionContext,
  commands,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind
} from 'vscode';

// This method is called when your extension is activated
export function activate(context: ExtensionContext) {
  // Use the console to output diagnostic information (console.log) and errors (console.error)
  // This line of code will only be executed once when your extension is activated
  console.log('Bend-PVM extension is now active');

  // The command has been defined in the package.json file
  // Now provide the implementation of the command with registerCommand
  // The commandId parameter must match the command field in package.json
  let disposable = commands.registerCommand('bend-pvm.activate', () => {
    showOutput();
    window.showInformationMessage('Bend-PVM Language Server activated!');
  });

  context.subscriptions.push(disposable);

  // Register restart command
  let restartCommand = commands.registerCommand('bend-pvm.restart', async () => {
    if (languageClient) {
      await languageClient.stop();
    }
    startServer(context);
    window.showInformationMessage('Bend-PVM Language Server restarted!');
  });

  context.subscriptions.push(restartCommand);

  // Register show output command
  let showOutputCommand = commands.registerCommand('bend-pvm.showOutput', () => {
    showOutput();
  });

  context.subscriptions.push(showOutputCommand);

  // Start the language server
  startServer(context);
}

let languageClient: LanguageClient | undefined;

function startServer(context: ExtensionContext) {
  // Configuration for the server
  const config = workspace.getConfiguration('bend-pvm');
  const serverPath = config.get<string>('server.path', '');
  const traceLevel = config.get<string>('trace.server', 'off');

  // If server path is provided, use it, otherwise look in common locations
  let serverExecutable: string;
  if (serverPath && serverPath.trim() !== '') {
    serverExecutable = serverPath;
  } else {
    // Try common locations
    serverExecutable = findServerExecutable();
  }

  // If we found the server, use it
  if (serverExecutable) {
    startLanguageServer(context, serverExecutable, traceLevel);
  } else {
    // Fall back to the bundled LSP server
    window.showWarningMessage(
      'Bend-PVM server not found. Please configure bend-pvm.server.path in settings.',
      'Configure'
    ).then(selection => {
      if (selection === 'Configure') {
        commands.executeCommand('workbench.action.openSettings', 'bend-pvm.server.path');
      }
    });
  }
}

function findServerExecutable(): string {
  const paths = [
    // Project-level
    'target/release/bend-pvm-lsp',
    'target/release/bend-pvm',
    'target/debug/bend-pvm-lsp',
    'target/debug/bend-pvm',
    
    // Global installations
    '/usr/local/bin/bend-pvm-lsp',
    '/usr/bin/bend-pvm-lsp',
    
    // Homebrew
    process.env.HOME + '/.brew/bin/bend-pvm-lsp',
    
    // NPM global
    process.env.npm_config_prefix + '/bin/bend-pvm-lsp',
  ];

  for (const path of paths) {
    try {
      const fs = require('fs');
      if (fs.existsSync(path)) {
        return path;
      }
    } catch (e) {
      // Ignore errors
    }
  }

  return '';
}

function startLanguageServer(context: ExtensionContext, serverPath: string, traceLevel: string) {
  // Configure the server options
  const serverOptions: ServerOptions = {
    run: {
      command: serverPath,
      transport: TransportKind.stdio
    },
    debug: {
      command: serverPath,
      transport: TransportKind.stdio,
      options: {
        env: {
          RUST_LOG: 'debug'
        }
      }
    }
  };

  // Configure the client options
  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      {
        scheme: 'file',
        language: 'bend'
      }
    ],
    synchronize: {
      fileEvents: [
        workspace.createFileSystemWatcher('**/*.bend')
      ]
    },
    traceOutputChannel: {
      name: 'Bend-PVM Trace',
      log: traceLevel === 'verbose',
      reveal: traceLevel === 'messages'
    },
    outputChannelName: 'Bend-PVM',
    revealOutputChannelOn: traceLevel === 'messages' 
      ? 3 // Error
      : 4 // Never
  };

  // Create the language client
  languageClient = new LanguageClient(
    'bend-pvm-lsp',
    'Bend-PVM Language Server',
    serverOptions,
    clientOptions
  );

  // Start the client and server
  languageClient.start();

  // Push the disposable
  context.subscriptions.push({
    dispose: () => {
      if (languageClient) {
        languageClient.stop();
      }
    }
  });
}

function showOutput() {
  if (languageClient) {
    languageClient.outputChannel.show();
  }
}
