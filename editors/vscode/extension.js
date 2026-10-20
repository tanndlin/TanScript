const vscode = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");

let client;

function activate() {
  const command =
    process.env.TANSCRIPT_LSP ?? vscode.workspace.getConfiguration("tanscript").get("serverPath");
  client = new LanguageClient(
    "tanscript",
    "TanScript",
    { command },
    { documentSelector: [{ scheme: "file", language: "tanscript" }] },
  );
  client.start();
}

function deactivate() {
  return client?.stop();
}

module.exports = { activate, deactivate };
