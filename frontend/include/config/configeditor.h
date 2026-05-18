// NewtonEmu Frontend - Qt 6 GUI for NewtonEmu
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#ifndef CONFIGEDITOR_H
#define CONFIGEDITOR_H

#include <QWidget>
#include <QString>

QT_BEGIN_NAMESPACE
namespace Ui { class ConfigEditor; }
QT_END_NAMESPACE

namespace NewtonEmu {

class ConfigModel;

class ConfigEditor : public QWidget
{
    Q_OBJECT

public:
    explicit ConfigEditor(QWidget *parent = nullptr);
    ~ConfigEditor();

    void newConfiguration();
    bool loadConfiguration(const QString &filePath);
    bool saveConfiguration(const QString &filePath);
    
    ConfigModel* model() const { return configModel; }

private slots:
    void onBrowseRom();
    void onBrowseBootCd();
    void onBrowseBootDisk();
    
    void onAddScsiDevice();
    void onRemoveScsiDevice();
    void onEditScsiDevice();
    
    void onAddIdeDevice();
    void onRemoveIdeDevice();
    void onEditIdeDevice();
    
    void onCpuModelChanged(int index);
    void onRamSizeChanged(int value);
    void onDisplayWidthChanged(int value);
    void onDisplayHeightChanged(int value);
    void onNetworkEnabledChanged(bool enabled);

private:
    void setupConnections();
    void updateFromModel();
    void updateModelFromUi();
    void populateStorageLists();
    
    Ui::ConfigEditor *ui;
    ConfigModel *configModel;
};

} // namespace NewtonEmu

#endif // CONFIGEDITOR_H
