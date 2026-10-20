const fs = require("fs");
const os = require("os");
const path = require("path");
const vscode = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");

let client;
let shadow;
let watched;
let restartTimer;

function shadowCopy(source) {
  const dest = path.join(os.tmpdir(), `tanscript-lsp-${process.pid}-${Date.now()}${path.extname(source)}`);
  fs.copyFileSync(source, dest);
  return dest;
}

function removeShadow() {
  if (shadow) {
    fs.rm(shadow, { force: true }, () => {});
    shadow = undefined;
  }
}

async function startClient(command) {
  if (fs.existsSync(command)) {
    shadow = shadowCopy(command);
    command = shadow;
  }
  client = new LanguageClient(
    "tanscript",
    "TanScript",
    { command },
    { documentSelector: [{ scheme: "file", language: "tanscript" }] },
  );
  await client.start();
}

async function restart(command) {
  await client?.stop();
  removeShadow();
  await startClient(command);
}

function activate() {
  const command =
    process.env.TANSCRIPT_LSP ?? vscode.workspace.getConfiguration("tanscript").get("serverPath");
  startClient(command);

  if (fs.existsSync(command)) {
    watched = command;
    fs.watchFile(command, { interval: 1000 }, (curr, prev) => {
      if (curr.mtimeMs === prev.mtimeMs || curr.size === 0) return;
      clearTimeout(restartTimer);
      restartTimer = setTimeout(() => restart(command), 500);
    });
  }
}

async function deactivate() {
  clearTimeout(restartTimer);
  if (watched) fs.unwatchFile(watched);
  await client?.stop();
  removeShadow();
}

module.exports = { activate, deactivate };
