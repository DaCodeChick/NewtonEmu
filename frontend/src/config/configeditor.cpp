// NewtonEmu Frontend - Qt 6 GUI for NewtonEmu
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#include "config/configeditor.h"
#include "ui_configeditor.h"
#include "config/configmodel.h"

#include <QFileDialog>
#include <QStandardPaths>
#include <QInputDialog>
#include <QMessageBox>
#include <QFile>

namespace NewtonEmu {

ConfigEditor::ConfigEditor(QWidget *parent)
    : QWidget(parent)
    , ui(new Ui::ConfigEditor)
    , configModel(new ConfigModel(this))
{
    ui->setupUi(this);
    setupConnections();
    updateFromModel();
}

ConfigEditor::~ConfigEditor()
{
    delete ui;
}

void ConfigEditor::newConfiguration()
{
    configModel->reset();
    updateFromModel();
}

bool ConfigEditor::loadConfiguration(const QString &filePath)
{
    if (configModel->loadFromFile(filePath)) {
        updateFromModel();
        return true;
    }
    return false;
}

bool ConfigEditor::saveConfiguration(const QString &filePath)
{
    updateModelFromUi();
    return configModel->saveToFile(filePath);
}

void ConfigEditor::refreshUi()
{
    updateFromModel();
}

void ConfigEditor::setupConnections()
{
    // ROM browser
    connect(ui->romBrowseButton, &QPushButton::clicked, this, &ConfigEditor::onBrowseRom);
    
    // CPU/Memory
    connect(ui->cpuModelCombo, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &ConfigEditor::onCpuModelChanged);
    connect(ui->ramSizeSpinBox, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ConfigEditor::onRamSizeChanged);
    
    // Display
    connect(ui->displayWidthSpinBox, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ConfigEditor::onDisplayWidthChanged);
    connect(ui->displayHeightSpinBox, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ConfigEditor::onDisplayHeightChanged);
    
    // Storage
    connect(ui->bootCdBrowseButton, &QPushButton::clicked, this, &ConfigEditor::onBrowseBootCd);
    connect(ui->bootDiskBrowseButton, &QPushButton::clicked, this, &ConfigEditor::onBrowseBootDisk);
    connect(ui->bootDiskCreateButton, &QPushButton::clicked, this, &ConfigEditor::onCreateBootDisk);
    
    // Network
    connect(ui->networkEnabledCheckBox, &QCheckBox::toggled,
            this, &ConfigEditor::onNetworkEnabledChanged);
}

void ConfigEditor::updateFromModel()
{
    // CPU
    ui->cpuModelCombo->setCurrentIndex(static_cast<int>(configModel->cpuModel()));
    ui->clockSpeedSpinBox->setValue(configModel->clockSpeed());
    
    // Memory
    ui->ramSizeSpinBox->setValue(configModel->ramSizeMb());
    ui->romPathEdit->setText(configModel->romPath());
    
    // Display
    ui->displayWidthSpinBox->setValue(configModel->displayWidth());
    ui->displayHeightSpinBox->setValue(configModel->displayHeight());
    
    // Storage
    ui->bootCdEdit->setText(configModel->bootCd());
    ui->bootDiskEdit->setText(configModel->bootDisk());
    
    // Network
    ui->networkEnabledCheckBox->setChecked(configModel->networkEnabled());
    ui->networkTypeCombo->setEnabled(configModel->networkEnabled());
}

void ConfigEditor::updateModelFromUi()
{
    // CPU
    configModel->setCpuModel(static_cast<CpuModel>(ui->cpuModelCombo->currentIndex()));
    configModel->setClockSpeed(ui->clockSpeedSpinBox->value());
    
    // Memory
    configModel->setRamSizeMb(ui->ramSizeSpinBox->value());
    configModel->setRomPath(ui->romPathEdit->text());
    
    // Display
    configModel->setDisplayWidth(ui->displayWidthSpinBox->value());
    configModel->setDisplayHeight(ui->displayHeightSpinBox->value());
    
    // Storage
    configModel->setBootCd(ui->bootCdEdit->text());
    configModel->setBootDisk(ui->bootDiskEdit->text());
    
    // Network
    configModel->setNetworkEnabled(ui->networkEnabledCheckBox->isChecked());
}

void ConfigEditor::onBrowseRom()
{
    QString fileName = QFileDialog::getOpenFileName(
        this,
        tr("Select ROM File"),
        QStandardPaths::writableLocation(QStandardPaths::HomeLocation) + "/Documents/GitHub/NewtonEmu/roms",
        tr("ROM Files (*.rom *.ROM);;All Files (*)")
    );
    
    if (!fileName.isEmpty()) {
        ui->romPathEdit->setText(fileName);
        configModel->setRomPath(fileName);
    }
}

void ConfigEditor::onBrowseBootCd()
{
    QString fileName = QFileDialog::getOpenFileName(
        this,
        tr("Select Boot CD/DVD Image"),
        QStandardPaths::writableLocation(QStandardPaths::HomeLocation) + "/Documents/GitHub/NewtonEmu/disks",
        tr("ISO Images (*.iso *.ISO);;All Files (*)")
    );
    
    if (!fileName.isEmpty()) {
        ui->bootCdEdit->setText(fileName);
        configModel->setBootCd(fileName);
    }
}

void ConfigEditor::onBrowseBootDisk()
{
    QString fileName = QFileDialog::getOpenFileName(
        this,
        tr("Select Boot Disk Image"),
        QStandardPaths::writableLocation(QStandardPaths::HomeLocation) + "/Documents/GitHub/NewtonEmu/disks",
        tr("Disk Images (*.img *.raw);;All Files (*)")
    );
    
    if (!fileName.isEmpty()) {
        ui->bootDiskEdit->setText(fileName);
        configModel->setBootDisk(fileName);
    }
}

void ConfigEditor::onCreateBootDisk()
{
    QString fileName = QFileDialog::getSaveFileName(
        this,
        tr("Create New Boot Disk Image"),
        QStandardPaths::writableLocation(QStandardPaths::HomeLocation) + "/Documents/GitHub/NewtonEmu/disks/new_disk.img",
        tr("Disk Images (*.img *.raw);;All Files (*)")
    );
    
    if (!fileName.isEmpty()) {
        // Ask for disk size
        bool ok;
        int sizeMb = QInputDialog::getInt(
            this,
            tr("Disk Size"),
            tr("Enter disk size in MB:"),
            1024, // default 1GB
            1,    // minimum 1MB
            1024 * 1024, // maximum 1TB
            1,    // step
            &ok
        );
        
        if (ok) {
            // Create the disk file directly
            QFile file(fileName);
            if (file.open(QIODevice::WriteOnly)) {
                qint64 sizeBytes = static_cast<qint64>(sizeMb) * 1024 * 1024;
                if (file.resize(sizeBytes)) {
                    file.close();
                    ui->bootDiskEdit->setText(fileName);
                    configModel->setBootDisk(fileName);
                    QMessageBox::information(this, tr("Success"), 
                        tr("Disk image created successfully: %1 MB").arg(sizeMb));
                } else {
                    file.close();
                    QMessageBox::warning(this, tr("Error"), 
                        tr("Failed to set disk image size"));
                }
            } else {
                QMessageBox::warning(this, tr("Error"), 
                    tr("Failed to create disk image file"));
            }
        }
    }
}


void ConfigEditor::onAddScsiDevice()
{
    // TODO: Implement
}

void ConfigEditor::onRemoveScsiDevice()
{
    // TODO: Implement
}

void ConfigEditor::onEditScsiDevice()
{
    // TODO: Implement
}

void ConfigEditor::onAddIdeDevice()
{
    // TODO: Implement
}

void ConfigEditor::onRemoveIdeDevice()
{
    // TODO: Implement
}

void ConfigEditor::onEditIdeDevice()
{
    // TODO: Implement
}

void ConfigEditor::onCpuModelChanged(int index)
{
    configModel->setCpuModel(static_cast<CpuModel>(index));
}

void ConfigEditor::onRamSizeChanged(int value)
{
    configModel->setRamSizeMb(value);
}

void ConfigEditor::onDisplayWidthChanged(int value)
{
    configModel->setDisplayWidth(value);
}

void ConfigEditor::onDisplayHeightChanged(int value)
{
    configModel->setDisplayHeight(value);
}

void ConfigEditor::onNetworkEnabledChanged(bool enabled)
{
    configModel->setNetworkEnabled(enabled);
    ui->networkTypeCombo->setEnabled(enabled);
}

} // namespace NewtonEmu
