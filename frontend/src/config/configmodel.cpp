// NewtonEmu Frontend - Qt 6 GUI for NewtonEmu
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#include "config/configmodel.h"
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QDebug>

namespace NewtonEmu {

ConfigModel::ConfigModel(QObject *parent)
    : QObject(parent)
    , m_cpuModel(CpuModel::G4_7400)
    , m_clockSpeed(450)
    , m_ramSizeMb(256)
    , m_displayWidth(800)
    , m_displayHeight(600)
    , m_colorDepth(32)
    , m_networkEnabled(false)
    , m_networkType("slirp")
    , m_macAddress("52:54:00:12:34:56")
{
}

void ConfigModel::setCpuModel(CpuModel model)
{
    if (m_cpuModel != model) {
        m_cpuModel = model;
        emit configChanged();
    }
}

void ConfigModel::setClockSpeed(int mhz)
{
    if (m_clockSpeed != mhz) {
        m_clockSpeed = mhz;
        emit configChanged();
    }
}

void ConfigModel::setRamSizeMb(int mb)
{
    if (m_ramSizeMb != mb) {
        m_ramSizeMb = mb;
        emit configChanged();
    }
}

void ConfigModel::setRomPath(const QString &path)
{
    if (m_romPath != path) {
        m_romPath = path;
        emit configChanged();
    }
}

void ConfigModel::setDisplayWidth(int width)
{
    if (m_displayWidth != width) {
        m_displayWidth = width;
        emit configChanged();
    }
}

void ConfigModel::setDisplayHeight(int height)
{
    if (m_displayHeight != height) {
        m_displayHeight = height;
        emit configChanged();
    }
}

void ConfigModel::setColorDepth(int depth)
{
    if (m_colorDepth != depth) {
        m_colorDepth = depth;
        emit configChanged();
    }
}

void ConfigModel::setBootCd(const QString &path)
{
    if (m_bootCd != path) {
        m_bootCd = path;
        emit configChanged();
    }
}

void ConfigModel::setBootDisk(const QString &path)
{
    if (m_bootDisk != path) {
        m_bootDisk = path;
        emit configChanged();
    }
}

void ConfigModel::addStorageDevice(const StorageDevice &device)
{
    m_storageDevices.append(device);
    emit configChanged();
}

void ConfigModel::removeStorageDevice(int index)
{
    if (index >= 0 && index < m_storageDevices.size()) {
        m_storageDevices.removeAt(index);
        emit configChanged();
    }
}

void ConfigModel::updateStorageDevice(int index, const StorageDevice &device)
{
    if (index >= 0 && index < m_storageDevices.size()) {
        m_storageDevices[index] = device;
        emit configChanged();
    }
}

void ConfigModel::setNetworkEnabled(bool enabled)
{
    if (m_networkEnabled != enabled) {
        m_networkEnabled = enabled;
        emit configChanged();
    }
}

void ConfigModel::setNetworkType(const QString &type)
{
    if (m_networkType != type) {
        m_networkType = type;
        emit configChanged();
    }
}

void ConfigModel::setMacAddress(const QString &mac)
{
    if (m_macAddress != mac) {
        m_macAddress = mac;
        emit configChanged();
    }
}

void ConfigModel::setGdbServer(const QString &addr)
{
    if (m_gdbServer != addr) {
        m_gdbServer = addr;
        emit configChanged();
    }
}

void ConfigModel::setIpcSocket(const QString &path)
{
    if (m_ipcSocket != path) {
        m_ipcSocket = path;
        emit configChanged();
    }
}

void ConfigModel::reset()
{
    m_cpuModel = CpuModel::G4_7400;
    m_clockSpeed = 450;
    m_ramSizeMb = 256;
    m_romPath.clear();
    m_displayWidth = 800;
    m_displayHeight = 600;
    m_colorDepth = 32;
    m_bootCd.clear();
    m_bootDisk.clear();
    m_storageDevices.clear();
    m_networkEnabled = false;
    m_networkType = "slirp";
    m_macAddress = "52:54:00:12:34:56";
    m_gdbServer.clear();
    m_ipcSocket.clear();
    
    emit configChanged();
}

QString ConfigModel::cpuModelToString(CpuModel model) const
{
    switch (model) {
        case CpuModel::G3_740: return "G3_740";
        case CpuModel::G3_750: return "G3_750";
        case CpuModel::G4_7400: return "G4_7400";
        case CpuModel::G4_7450: return "G4_7450";
    }
    return "G4_7400";
}

CpuModel ConfigModel::stringToCpuModel(const QString &str) const
{
    if (str == "G3_740") return CpuModel::G3_740;
    if (str == "G3_750") return CpuModel::G3_750;
    if (str == "G4_7400") return CpuModel::G4_7400;
    if (str == "G4_7450") return CpuModel::G4_7450;
    return CpuModel::G4_7400;
}

bool ConfigModel::saveToFile(const QString &filePath)
{
    QFile file(filePath);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        qWarning() << "Failed to open file for writing:" << filePath;
        return false;
    }
    
    QJsonObject root;
    
    // CPU section
    QJsonObject cpu;
    cpu["model"] = cpuModelToString(m_cpuModel);
    cpu["clock_speed"] = m_clockSpeed;
    root["cpu"] = cpu;
    
    // Memory section
    QJsonObject memory;
    memory["ram_size_mb"] = m_ramSizeMb;
    if (!m_romPath.isEmpty()) {
        memory["rom_path"] = m_romPath;
    }
    root["memory"] = memory;
    
    // Display section
    QJsonObject display;
    display["width"] = m_displayWidth;
    display["height"] = m_displayHeight;
    display["color_depth"] = m_colorDepth;
    root["display"] = display;
    
    // Storage section
    QJsonObject storage;
    if (!m_bootCd.isEmpty()) {
        storage["boot_cd"] = m_bootCd;
    }
    if (!m_bootDisk.isEmpty()) {
        storage["boot_disk"] = m_bootDisk;
    }
    
    // SCSI devices
    QJsonArray scsiDevices;
    for (const auto &dev : m_storageDevices) {
        if (dev.bus == StorageDevice::Bus::SCSI) {
            QJsonObject scsiDev;
            scsiDev["id"] = dev.id;
            scsiDev["path"] = dev.path;
            if (dev.readonly) {
                scsiDev["readonly"] = true;
            }
            scsiDevices.append(scsiDev);
        }
    }
    if (!scsiDevices.isEmpty()) {
        storage["scsi"] = scsiDevices;
    }
    
    // IDE devices
    QJsonArray ideDevices;
    for (const auto &dev : m_storageDevices) {
        if (dev.bus == StorageDevice::Bus::IDE) {
            QJsonObject ideDev;
            ideDev["channel"] = dev.id;
            ideDev["device"] = dev.device;
            ideDev["path"] = dev.path;
            ideDevices.append(ideDev);
        }
    }
    if (!ideDevices.isEmpty()) {
        storage["ide"] = ideDevices;
    }
    
    if (!storage.isEmpty()) {
        root["storage"] = storage;
    }
    
    // Network section
    QJsonObject network;
    network["enabled"] = m_networkEnabled;
    if (m_networkEnabled) {
        network["type"] = m_networkType;
        network["mac_address"] = m_macAddress;
    }
    root["network"] = network;
    
    // Debug section
    QJsonObject debug;
    if (!m_gdbServer.isEmpty()) {
        debug["gdb_server"] = m_gdbServer;
    }
    if (!m_ipcSocket.isEmpty()) {
        debug["ipc_socket"] = m_ipcSocket;
    }
    if (!debug.isEmpty()) {
        root["debug"] = debug;
    }
    
    // Write JSON to file
    QJsonDocument doc(root);
    file.write(doc.toJson(QJsonDocument::Indented));
    file.close();
    
    return true;
}

bool ConfigModel::loadFromFile(const QString &filePath)
{
    QFile file(filePath);
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        qWarning() << "Failed to open file for reading:" << filePath;
        return false;
    }
    
    QByteArray data = file.readAll();
    file.close();
    
    QJsonParseError parseError;
    QJsonDocument doc = QJsonDocument::fromJson(data, &parseError);
    
    if (parseError.error != QJsonParseError::NoError) {
        qWarning() << "JSON parse error:" << parseError.errorString();
        return false;
    }
    
    if (!doc.isObject()) {
        qWarning() << "JSON root is not an object";
        return false;
    }
    
    QJsonObject root = doc.object();
    
    // Parse CPU
    if (root.contains("cpu")) {
        QJsonObject cpu = root["cpu"].toObject();
        if (cpu.contains("model")) {
            m_cpuModel = stringToCpuModel(cpu["model"].toString());
        }
        if (cpu.contains("clock_speed")) {
            m_clockSpeed = cpu["clock_speed"].toInt();
        }
    }
    
    // Parse Memory
    if (root.contains("memory")) {
        QJsonObject memory = root["memory"].toObject();
        if (memory.contains("ram_size_mb")) {
            m_ramSizeMb = memory["ram_size_mb"].toInt();
        }
        if (memory.contains("rom_path")) {
            m_romPath = memory["rom_path"].toString();
        }
    }
    
    // Parse Display
    if (root.contains("display")) {
        QJsonObject display = root["display"].toObject();
        if (display.contains("width")) {
            m_displayWidth = display["width"].toInt();
        }
        if (display.contains("height")) {
            m_displayHeight = display["height"].toInt();
        }
        if (display.contains("color_depth")) {
            m_colorDepth = display["color_depth"].toInt();
        }
    }
    
    // Parse Storage
    if (root.contains("storage")) {
        QJsonObject storage = root["storage"].toObject();
        
        if (storage.contains("boot_cd")) {
            m_bootCd = storage["boot_cd"].toString();
        }
        if (storage.contains("boot_disk")) {
            m_bootDisk = storage["boot_disk"].toString();
        }
        
        // Parse SCSI devices
        if (storage.contains("scsi")) {
            QJsonArray scsi = storage["scsi"].toArray();
            for (const QJsonValue &val : scsi) {
                QJsonObject scsiDev = val.toObject();
                StorageDevice dev;
                dev.bus = StorageDevice::Bus::SCSI;
                dev.type = StorageDevice::Type::HardDisk;
                dev.id = scsiDev["id"].toInt();
                dev.path = scsiDev["path"].toString();
                dev.readonly = scsiDev.value("readonly").toBool(false);
                m_storageDevices.append(dev);
            }
        }
        
        // Parse IDE devices
        if (storage.contains("ide")) {
            QJsonArray ide = storage["ide"].toArray();
            for (const QJsonValue &val : ide) {
                QJsonObject ideDev = val.toObject();
                StorageDevice dev;
                dev.bus = StorageDevice::Bus::IDE;
                dev.type = StorageDevice::Type::HardDisk;
                dev.id = ideDev["channel"].toInt();
                dev.device = ideDev["device"].toInt();
                dev.path = ideDev["path"].toString();
                m_storageDevices.append(dev);
            }
        }
    }
    
    // Parse Network
    if (root.contains("network")) {
        QJsonObject network = root["network"].toObject();
        if (network.contains("enabled")) {
            m_networkEnabled = network["enabled"].toBool();
        }
        if (network.contains("type")) {
            m_networkType = network["type"].toString();
        }
        if (network.contains("mac_address")) {
            m_macAddress = network["mac_address"].toString();
        }
    }
    
    // Parse Debug
    if (root.contains("debug")) {
        QJsonObject debug = root["debug"].toObject();
        if (debug.contains("gdb_server")) {
            m_gdbServer = debug["gdb_server"].toString();
        }
        if (debug.contains("ipc_socket")) {
            m_ipcSocket = debug["ipc_socket"].toString();
        }
    }
    
    emit configChanged();
    return true;
}

} // namespace NewtonEmu
