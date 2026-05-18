// NewtonEmu Frontend - Qt 6 GUI for NewtonEmu
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#include "mainwindow.h"
#include <QApplication>
#include <QCommandLineParser>
#include <QDebug>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    
    // Application metadata
    QCoreApplication::setApplicationName("NewtonEmu Frontend");
    QCoreApplication::setApplicationVersion("0.1.0");
    QCoreApplication::setOrganizationName("NewtonEmu");
    QCoreApplication::setOrganizationDomain("newtonemu.org");
    
    // Command-line parsing
    QCommandLineParser parser;
    parser.setApplicationDescription("Qt 6 Frontend for NewtonEmu PowerPC Macintosh Emulator");
    parser.addHelpOption();
    parser.addVersionOption();
    
    QCommandLineOption configOption(
        QStringList() << "c" << "config",
        "Load configuration file",
        "file"
    );
    parser.addOption(configOption);
    
    QCommandLineOption emulatorOption(
        QStringList() << "e" << "emulator",
        "Path to newton-emu binary",
        "path"
    );
    parser.addOption(emulatorOption);
    
    parser.process(app);
    
    // Create and show main window
    NewtonEmu::MainWindow window;
    
    // Load config if specified
    if (parser.isSet(configOption)) {
        QString configFile = parser.value(configOption);
        qInfo() << "Loading configuration from:" << configFile;
        // TODO: Load config
    }
    
    // Set emulator path if specified
    if (parser.isSet(emulatorOption)) {
        QString emulatorPath = parser.value(emulatorOption);
        qInfo() << "Using emulator binary:" << emulatorPath;
        // TODO: Set emulator path
    }
    
    window.show();
    
    return app.exec();
}
