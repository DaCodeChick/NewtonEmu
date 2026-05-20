// NewtonEmu - Qt 6 GUI for NewtonEmu
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#include "mainwindow.h"
#include "ui_mainwindow.h"
#include "config/configeditor.h"
#include "config/configmodel.h"

#include <QFileDialog>
#include <QMessageBox>
#include <QStandardPaths>
#include <QDir>
#include <QFileInfo>
#include <QDebug>

namespace NewtonEmu {

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
    , configEditor(nullptr)
    , emulatorProcess(nullptr)
    , emulatorBinaryPath(findEmulatorBinary())
{
    ui->setupUi(this);
    
    // Create config editor
    configEditor = new ConfigEditor(this);
    setCentralWidget(configEditor);
    
    // Load config from default location
    configEditor->model()->loadOrDefault();
    configEditor->refreshUi();
    
    // Auto-save on config changes
    connect(configEditor->model(), &ConfigModel::configChanged, 
            this, &MainWindow::onConfigChanged);
    
    // Create UI elements
    createActions();
    createMenus();
    createToolBar();
    createStatusBar();
    
    // Set window title
    setWindowTitle(tr("NewtonEmu"));
    
    // Initial state
    updateEmulatorControls();
    
    // Show emulator path in status
    if (!emulatorBinaryPath.isEmpty()) {
        statusBar()->showMessage(tr("Emulator: %1").arg(emulatorBinaryPath));
    } else {
        statusBar()->showMessage(tr("Emulator binary not found - please build the emulator first"), 5000);
    }
}

MainWindow::~MainWindow()
{
    if (emulatorProcess && emulatorProcess->state() != QProcess::NotRunning) {
        emulatorProcess->terminate();
        emulatorProcess->waitForFinished(3000);
    }
    delete ui;
}

void MainWindow::createActions()
{
    // File menu actions are created in the UI file
    connect(ui->actionNew, &QAction::triggered, this, &MainWindow::onNewConfiguration);
    connect(ui->actionOpen, &QAction::triggered, this, &MainWindow::onOpenConfiguration);
    connect(ui->actionSave, &QAction::triggered, this, &MainWindow::onSaveConfiguration);
    connect(ui->actionSaveAs, &QAction::triggered, this, &MainWindow::onSaveConfigurationAs);
    connect(ui->actionExit, &QAction::triggered, this, &MainWindow::onExit);
    
    // Emulator menu
    connect(ui->actionLaunch, &QAction::triggered, this, &MainWindow::onLaunchEmulator);
    connect(ui->actionStop, &QAction::triggered, this, &MainWindow::onStopEmulator);
    
    // Tools menu
    connect(ui->actionDebugger, &QAction::triggered, this, &MainWindow::onOpenDebugger);
    connect(ui->actionHfsManager, &QAction::triggered, this, &MainWindow::onOpenHfsManager);
    connect(ui->actionNetworkMonitor, &QAction::triggered, this, &MainWindow::onOpenNetworkMonitor);
    connect(ui->actionDiskManager, &QAction::triggered, this, &MainWindow::onOpenDiskManager);
    
    // Help menu
    connect(ui->actionAbout, &QAction::triggered, this, &MainWindow::onAbout);
    connect(ui->actionAboutQt, &QAction::triggered, this, &MainWindow::onAboutQt);
}

void MainWindow::createMenus()
{
    // Menus are created in the UI file
}

void MainWindow::createToolBar()
{
    // Toolbar is created in the UI file
}

void MainWindow::createStatusBar()
{
    statusBar()->showMessage(tr("Ready"));
}

void MainWindow::onNewConfiguration()
{
    configEditor->newConfiguration();
    currentConfigFile.clear();
    setWindowTitle(tr("NewtonEmu"));
    statusBar()->showMessage(tr("New configuration created"), 2000);
}

void MainWindow::onOpenConfiguration()
{
    QString fileName = QFileDialog::getOpenFileName(
        this,
        tr("Open Configuration"),
        QStandardPaths::writableLocation(QStandardPaths::HomeLocation) + "/.config/newton-emu",
        tr("JSON Configuration Files (*.json);;All Files (*)")
    );
    
    if (!fileName.isEmpty()) {
        if (configEditor->loadConfiguration(fileName)) {
            currentConfigFile = fileName;
            setWindowTitle(tr("NewtonEmu"));
            statusBar()->showMessage(tr("Configuration loaded: %1").arg(fileName), 3000);
        } else {
            QMessageBox::warning(this, tr("Error"), tr("Failed to load configuration file"));
        }
    }
}

void MainWindow::onSaveConfiguration()
{
    // Auto-save to default location
    configEditor->model()->saveDefault();
    statusBar()->showMessage(tr("Configuration saved"), 2000);
}

void MainWindow::onConfigChanged()
{
    // Auto-save to default location on any config change
    configEditor->model()->saveDefault();
}

void MainWindow::onSaveConfigurationAs()
{
    QString fileName = QFileDialog::getSaveFileName(
        this,
        tr("Save Configuration"),
        QStandardPaths::writableLocation(QStandardPaths::HomeLocation) + "/.config/newton-emu/config.json",
        tr("JSON Configuration Files (*.json);;All Files (*)")
    );
    
    if (!fileName.isEmpty()) {
        if (configEditor->saveConfiguration(fileName)) {
            currentConfigFile = fileName;
            setWindowTitle(tr("NewtonEmu"));
            statusBar()->showMessage(tr("Configuration saved: %1").arg(fileName), 3000);
        } else {
            QMessageBox::warning(this, tr("Error"), tr("Failed to save configuration file"));
        }
    }
}

void MainWindow::onExit()
{
    close();
}

void MainWindow::onLaunchEmulator()
{
    if (emulatorBinaryPath.isEmpty()) {
        QMessageBox::critical(
            this,
            tr("Error"),
            tr("Emulator binary not found.\n\nPlease build the emulator first:\n"
               "  cd ..\n"
               "  cargo build --release")
        );
        return;
    }
    
    // Launch emulator with current configuration
    if (launchEmulator()) {
        statusBar()->showMessage(tr("Emulator launched"), 2000);
    }
}

void MainWindow::onStopEmulator()
{
    if (emulatorProcess && emulatorProcess->state() != QProcess::NotRunning) {
        emulatorProcess->terminate();
        statusBar()->showMessage(tr("Stopping emulator..."), 2000);
    }
}

void MainWindow::onOpenDebugger()
{
    QMessageBox::information(this, tr("Debugger"), tr("Debugger not yet implemented"));
    // TODO: Open debugger window
}

void MainWindow::onOpenHfsManager()
{
    QMessageBox::information(this, tr("HFS+ Manager"), tr("HFS+ Manager not yet implemented"));
    // TODO: Open HFS+ manager window
}

void MainWindow::onOpenNetworkMonitor()
{
    QMessageBox::information(this, tr("Network Monitor"), tr("Network Monitor not yet implemented"));
    // TODO: Open network monitor window
}

void MainWindow::onOpenDiskManager()
{
    QMessageBox::information(this, tr("Disk Manager"), tr("Disk Manager not yet implemented"));
    // TODO: Open disk manager window
}

void MainWindow::onAbout()
{
    QMessageBox::about(
        this,
        tr("About NewtonEmu"),
        tr("<h2>NewtonEmu 0.1.0</h2>"
           "<p>Qt 6 GUI for NewtonEmu PowerPC Macintosh Emulator</p>"
           "<p>Copyright (C) 2026 NewtonEmu Contributors</p>"
           "<p>Licensed under GPL v3</p>"
           "<p><b>Features:</b></p>"
           "<ul>"
           "<li>Configuration Editor</li>"
           "<li>Live Debugger (coming soon)</li>"
           "<li>HFS+ File Manager (coming soon)</li>"
           "<li>Network Monitor (coming soon)</li>"
           "</ul>")
    );
}

void MainWindow::onAboutQt()
{
    QMessageBox::aboutQt(this, tr("About Qt"));
}

void MainWindow::updateEmulatorControls()
{
    bool running = emulatorProcess && emulatorProcess->state() != QProcess::NotRunning;
    
    ui->actionLaunch->setEnabled(!running && !emulatorBinaryPath.isEmpty());
    ui->actionStop->setEnabled(running);
}

QString MainWindow::findEmulatorBinary()
{
    // Try to find the emulator binary relative to the frontend
    // Frontend is at: frontend/build/bin/NewtonEmu
    // Need to go up 3 levels to reach project root
    QStringList searchPaths = {
        "../../../target/release/newton-emu",
        "../../../target/debug/newton-emu",
        "../../target/release/newton-emu",
        "../../target/debug/newton-emu",
        "../target/release/newton-emu",
        "../target/debug/newton-emu",
        "./newton-emu",
        "/usr/local/bin/newton-emu",
        "/usr/bin/newton-emu"
    };
    
    for (const QString &path : searchPaths) {
        QFileInfo info(path);
        if (info.exists() && info.isExecutable()) {
            return info.absoluteFilePath();
        }
    }
    
    return QString();
}

bool MainWindow::launchEmulator()
{
    if (emulatorProcess) {
        delete emulatorProcess;
    }
    
    emulatorProcess = new QProcess(this);
    
    connect(emulatorProcess, &QProcess::started, this, &MainWindow::onEmulatorStarted);
    connect(emulatorProcess, &QProcess::finished, this, &MainWindow::onEmulatorFinished);
    connect(emulatorProcess, &QProcess::errorOccurred, this, &MainWindow::onEmulatorError);
    connect(emulatorProcess, &QProcess::readyReadStandardOutput, this, &MainWindow::onEmulatorOutput);
    
    // Build command-line arguments from current configuration
    QStringList arguments;
    
    // Get config model from editor
    auto config = configEditor->model();
    
    // RAM
    if (config->ramSizeMb() != 256) {  // Only pass if not default
        arguments << "--ram" << QString::number(config->ramSizeMb());
    }
    
    // ROM
    if (!config->romPath().isEmpty()) {
        arguments << "--rom" << config->romPath();
    }
    
    // Display
    if (config->displayWidth() != 800 || config->displayHeight() != 600) {
        arguments << "--width" << QString::number(config->displayWidth());
        arguments << "--height" << QString::number(config->displayHeight());
    }
    
    // Boot CD/Disk
    if (!config->bootCd().isEmpty()) {
        arguments << "--cd" << config->bootCd();
    }
    if (!config->bootDisk().isEmpty()) {
        arguments << "--disk" << config->bootDisk();
    }
    
    // GDB server (for built-in debugger communication)
    if (!config->gdbServer().isEmpty()) {
        arguments << "--gdb-server" << config->gdbServer();
    }
    
    emulatorProcess->start(emulatorBinaryPath, arguments);
    
    return emulatorProcess->waitForStarted(5000);
}

void MainWindow::onEmulatorStarted()
{
    qInfo() << "Emulator started";
    updateEmulatorControls();
    statusBar()->showMessage(tr("Emulator running"), 3000);
}

void MainWindow::onEmulatorFinished(int exitCode, QProcess::ExitStatus exitStatus)
{
    qInfo() << "Emulator finished with exit code:" << exitCode;
    updateEmulatorControls();
    
    if (exitStatus == QProcess::CrashExit) {
        statusBar()->showMessage(tr("Emulator crashed"), 5000);
        QMessageBox::warning(this, tr("Emulator Crashed"), tr("The emulator has crashed"));
    } else {
        statusBar()->showMessage(tr("Emulator exited"), 3000);
    }
}

void MainWindow::onEmulatorError(QProcess::ProcessError error)
{
    QString errorMsg;
    switch (error) {
        case QProcess::FailedToStart:
            errorMsg = tr("Failed to start emulator");
            break;
        case QProcess::Crashed:
            errorMsg = tr("Emulator crashed");
            break;
        default:
            errorMsg = tr("Emulator error: %1").arg(static_cast<int>(error));
    }
    
    qWarning() << "Emulator error:" << errorMsg;
    statusBar()->showMessage(errorMsg, 5000);
    updateEmulatorControls();
}

void MainWindow::onEmulatorOutput()
{
    if (emulatorProcess) {
        QByteArray output = emulatorProcess->readAllStandardOutput();
        qDebug() << "Emulator output:" << output;
        // TODO: Show in log window
    }
}

} // namespace NewtonEmu
