import * as vscode from 'vscode';
import * as lc from 'vscode-languageclient';

import * as commands from './commands';
import { TextDocumentContentProvider } from './commands/syntaxTree';
import * as events from './events';
import * as notifications from './notifications';
import { Server } from './server';

export function activate(context: vscode.ExtensionContext) {
    function disposeOnDeactivation(disposable: vscode.Disposable) {
        context.subscriptions.push(disposable);
    }

    function registerCommand(name: string, f: any) {
        disposeOnDeactivation(vscode.commands.registerCommand(name, f));
    }

    // Commands are requests from vscode to the language server
    registerCommand('tom-lsp.syntaxTree', commands.syntaxTree.handle);
    registerCommand('tom-lsp.extendSelection', commands.extendSelection.handle);

    // Notifications are events triggered by the language server
    const allNotifications: Iterable<
        [string, lc.GenericNotificationHandler]
    > = [['m/publishDecorations', notifications.publishDecorations.handle]];

    // The events below are plain old javascript events, triggered and handled by vscode
    vscode.window.onDidChangeActiveTextEditor(
        events.changeActiveTextEditor.handle
    );

    const textDocumentContentProvider = new TextDocumentContentProvider();
    disposeOnDeactivation(
        vscode.workspace.registerTextDocumentContentProvider(
            'tom-lsp',
            textDocumentContentProvider
        )
    );

    vscode.workspace.onDidChangeTextDocument(
        events.changeTextDocument.createHandler(textDocumentContentProvider),
        null,
        context.subscriptions
    );

    // Start the language server, finally!
    Server.start(allNotifications);
}

export function deactivate(): Thenable<void> {
    if (!Server.client) {
        return Promise.resolve();
    }
    return Server.client.stop();
}
