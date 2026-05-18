// NewtonEmu Frontend - Qt 6 GUI for NewtonEmu
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QString>
#include <QProcess>

QT_BEGIN_NAMESPACE
namespace Ui { class MainWindow; }
QT_END_NAMESPACE

namespace NewtonEmu {

class ConfigEditor;

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

private slots:
    // Menu actions
    void onNewConfiguration();
    void onOpenConfiguration();
    void onSaveConfiguration();
    void onSaveConfigurationAs();
    void onExit();
    
    // Emulator actions
    void onLaunchEmulator();
    void onStopEmulator();
    
    // Tool actions
    void onOpenDebugger();
    void onOpenHfsManager();
    void onOpenNetworkMonitor();
    void onOpenDiskManager();
    
    // Help actions
    void onAbout();
    void onAboutQt();
    
    // Emulator process
    void onEmulatorStarted();
    void onEmulatorFinished(int exitCode, QProcess::ExitStatus exitStatus);
    void onEmulatorError(QProcess::ProcessError error);
    void onEmulatorOutput();

private:
    void createActions();
    void createMenus();
    void createToolBar();
    void createStatusBar();
    void updateEmulatorControls();
    
    QString findEmulatorBinary();
    bool launchEmulator();
    
    Ui::MainWindow *ui;
    ConfigEditor *configEditor;
    QProcess *emulatorProcess;
    QString currentConfigFile;
    QString emulatorBinaryPath;
};

} // namespace NewtonEmu

#endif // MAINWINDOW_H
