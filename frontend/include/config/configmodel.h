// NewtonEmu Frontend - Qt 6 GUI for NewtonEmu
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#ifndef CONFIGMODEL_H
#define CONFIGMODEL_H

#include <QObject>
#include <QString>
#include <QList>
#include <QVariantMap>
#include <QJsonObject>

namespace NewtonEmu {

enum class CpuModel {
    G3_740,
    G3_750,
    G4_7400,
    G4_7450
};

struct StorageDevice {
    enum class Type { HardDisk, CdRom };
    enum class Bus { SCSI, IDE };
    
    Type type;
    Bus bus;
    int id;              // SCSI ID or IDE channel
    int device;          // For IDE: 0=master, 1=slave
    QString path;
    bool readonly;
    
    StorageDevice()
        : type(Type::HardDisk)
        , bus(Bus::SCSI)
        , id(0)
        , device(0)
        , readonly(false)
    {}
};

class ConfigModel : public QObject
{
    Q_OBJECT

public:
    explicit ConfigModel(QObject *parent = nullptr);
    
    // CPU configuration
    CpuModel cpuModel() const { return m_cpuModel; }
    void setCpuModel(CpuModel model);
    
    int clockSpeed() const { return m_clockSpeed; }
    void setClockSpeed(int mhz);
    
    // Memory configuration
    int ramSizeMb() const { return m_ramSizeMb; }
    void setRamSizeMb(int mb);
    
    QString romPath() const { return m_romPath; }
    void setRomPath(const QString &path);
    
    // Display configuration
    int displayWidth() const { return m_displayWidth; }
    void setDisplayWidth(int width);
    
    int displayHeight() const { return m_displayHeight; }
    void setDisplayHeight(int height);
    
    int colorDepth() const { return m_colorDepth; }
    void setColorDepth(int depth);
    
    // Storage configuration
    QString bootCd() const { return m_bootCd; }
    void setBootCd(const QString &path);
    
    QString bootDisk() const { return m_bootDisk; }
    void setBootDisk(const QString &path);
    
    const QList<StorageDevice>& storageDevices() const { return m_storageDevices; }
    void addStorageDevice(const StorageDevice &device);
    void removeStorageDevice(int index);
    void updateStorageDevice(int index, const StorageDevice &device);
    
    // Network configuration
    bool networkEnabled() const { return m_networkEnabled; }
    void setNetworkEnabled(bool enabled);
    
    QString networkType() const { return m_networkType; }
    void setNetworkType(const QString &type);
    
    QString macAddress() const { return m_macAddress; }
    void setMacAddress(const QString &mac);
    
    // Emulator configuration
    QString emulatorPath() const { return m_emulatorPath; }
    void setEmulatorPath(const QString &path);
    
    // Load/Save
    bool loadFromFile(const QString &filePath);
    bool saveToFile(const QString &filePath);
    
    // Get default config file path (~/.config/newton-emu/config.json)
    static QString defaultConfigPath();
    
    // Load from default location or return default config
    bool loadOrDefault();
    
    // Save to default location
    bool saveDefault();
    
    void reset();

signals:
    void configChanged();

private:
    QString cpuModelToString(CpuModel model) const;
    CpuModel stringToCpuModel(const QString &str) const;
    
    QVariantMap toVariantMap() const;
    void fromVariantMap(const QVariantMap &map);
    
    // CPU
    CpuModel m_cpuModel;
    int m_clockSpeed;
    
    // Memory
    int m_ramSizeMb;
    QString m_romPath;
    
    // Display
    int m_displayWidth;
    int m_displayHeight;
    int m_colorDepth;
    
    // Storage
    QString m_bootCd;
    QString m_bootDisk;
    QList<StorageDevice> m_storageDevices;
    
    // Network
    bool m_networkEnabled;
    QString m_networkType;
    QString m_macAddress;
    
    // Emulator
    QString m_emulatorPath;
    
    // Preserve unknown sections (e.g., debug section used by backend)
    QJsonObject m_unknownSections;
};

} // namespace NewtonEmu

#endif // CONFIGMODEL_H
