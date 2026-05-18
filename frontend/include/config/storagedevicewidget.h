// NewtonEmu Frontend - Qt 6 GUI for NewtonEmu
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#ifndef STORAGEDEVICEWIDGET_H
#define STORAGEDEVICEWIDGET_H

#include <QWidget>

namespace NewtonEmu {

class StorageDeviceWidget : public QWidget
{
    Q_OBJECT

public:
    explicit StorageDeviceWidget(QWidget *parent = nullptr);
    ~StorageDeviceWidget();

    // TODO: Implement storage device management
};

} // namespace NewtonEmu

#endif // STORAGEDEVICEWIDGET_H
