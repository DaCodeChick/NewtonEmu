/********************************************************************************
** Form generated from reading UI file 'configeditor.ui'
**
** Created by: Qt User Interface Compiler version 6.11.1
**
** WARNING! All changes made in this file will be lost when recompiling UI file!
********************************************************************************/

#ifndef UI_CONFIGEDITOR_H
#define UI_CONFIGEDITOR_H

#include <QtCore/QVariant>
#include <QtWidgets/QApplication>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QGroupBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpacerItem>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

QT_BEGIN_NAMESPACE

class Ui_ConfigEditor
{
public:
    QVBoxLayout *verticalLayout;
    QTabWidget *tabWidget;
    QWidget *tabGeneral;
    QFormLayout *formLayout;
    QGroupBox *cpuGroupBox;
    QFormLayout *formLayout_2;
    QLabel *label;
    QComboBox *cpuModelCombo;
    QLabel *label_2;
    QSpinBox *clockSpeedSpinBox;
    QLabel *label_3;
    QSpinBox *ramSizeSpinBox;
    QGroupBox *romGroupBox;
    QHBoxLayout *horizontalLayout;
    QLineEdit *romPathEdit;
    QPushButton *romBrowseButton;
    QGroupBox *displayGroupBox;
    QFormLayout *formLayout_3;
    QLabel *label_4;
    QSpinBox *displayWidthSpinBox;
    QLabel *label_5;
    QSpinBox *displayHeightSpinBox;
    QWidget *tabStorage;
    QVBoxLayout *verticalLayout_2;
    QGroupBox *bootGroupBox;
    QFormLayout *formLayout_4;
    QLabel *label_6;
    QHBoxLayout *horizontalLayout_2;
    QLineEdit *bootCdEdit;
    QPushButton *bootCdBrowseButton;
    QLabel *label_7;
    QHBoxLayout *horizontalLayout_3;
    QLineEdit *bootDiskEdit;
    QPushButton *bootDiskBrowseButton;
    QLabel *label_8;
    QSpacerItem *verticalSpacer;
    QWidget *tabNetwork;
    QVBoxLayout *verticalLayout_3;
    QCheckBox *networkEnabledCheckBox;
    QGroupBox *networkSettingsGroupBox;
    QFormLayout *formLayout_5;
    QLabel *label_9;
    QComboBox *networkTypeCombo;
    QSpacerItem *verticalSpacer_2;

    void setupUi(QWidget *ConfigEditor)
    {
        if (ConfigEditor->objectName().isEmpty())
            ConfigEditor->setObjectName("ConfigEditor");
        ConfigEditor->resize(700, 600);
        verticalLayout = new QVBoxLayout(ConfigEditor);
        verticalLayout->setObjectName("verticalLayout");
        tabWidget = new QTabWidget(ConfigEditor);
        tabWidget->setObjectName("tabWidget");
        tabGeneral = new QWidget();
        tabGeneral->setObjectName("tabGeneral");
        formLayout = new QFormLayout(tabGeneral);
        formLayout->setObjectName("formLayout");
        cpuGroupBox = new QGroupBox(tabGeneral);
        cpuGroupBox->setObjectName("cpuGroupBox");
        formLayout_2 = new QFormLayout(cpuGroupBox);
        formLayout_2->setObjectName("formLayout_2");
        label = new QLabel(cpuGroupBox);
        label->setObjectName("label");

        formLayout_2->setWidget(0, QFormLayout::ItemRole::LabelRole, label);

        cpuModelCombo = new QComboBox(cpuGroupBox);
        cpuModelCombo->addItem(QString());
        cpuModelCombo->addItem(QString());
        cpuModelCombo->addItem(QString());
        cpuModelCombo->addItem(QString());
        cpuModelCombo->setObjectName("cpuModelCombo");

        formLayout_2->setWidget(0, QFormLayout::ItemRole::FieldRole, cpuModelCombo);

        label_2 = new QLabel(cpuGroupBox);
        label_2->setObjectName("label_2");

        formLayout_2->setWidget(1, QFormLayout::ItemRole::LabelRole, label_2);

        clockSpeedSpinBox = new QSpinBox(cpuGroupBox);
        clockSpeedSpinBox->setObjectName("clockSpeedSpinBox");
        clockSpeedSpinBox->setMinimum(100);
        clockSpeedSpinBox->setMaximum(2000);
        clockSpeedSpinBox->setValue(450);

        formLayout_2->setWidget(1, QFormLayout::ItemRole::FieldRole, clockSpeedSpinBox);

        label_3 = new QLabel(cpuGroupBox);
        label_3->setObjectName("label_3");

        formLayout_2->setWidget(2, QFormLayout::ItemRole::LabelRole, label_3);

        ramSizeSpinBox = new QSpinBox(cpuGroupBox);
        ramSizeSpinBox->setObjectName("ramSizeSpinBox");
        ramSizeSpinBox->setMinimum(64);
        ramSizeSpinBox->setMaximum(2048);
        ramSizeSpinBox->setSingleStep(64);
        ramSizeSpinBox->setValue(256);

        formLayout_2->setWidget(2, QFormLayout::ItemRole::FieldRole, ramSizeSpinBox);


        formLayout->setWidget(0, QFormLayout::ItemRole::SpanningRole, cpuGroupBox);

        romGroupBox = new QGroupBox(tabGeneral);
        romGroupBox->setObjectName("romGroupBox");
        horizontalLayout = new QHBoxLayout(romGroupBox);
        horizontalLayout->setObjectName("horizontalLayout");
        romPathEdit = new QLineEdit(romGroupBox);
        romPathEdit->setObjectName("romPathEdit");

        horizontalLayout->addWidget(romPathEdit);

        romBrowseButton = new QPushButton(romGroupBox);
        romBrowseButton->setObjectName("romBrowseButton");

        horizontalLayout->addWidget(romBrowseButton);


        formLayout->setWidget(1, QFormLayout::ItemRole::SpanningRole, romGroupBox);

        displayGroupBox = new QGroupBox(tabGeneral);
        displayGroupBox->setObjectName("displayGroupBox");
        formLayout_3 = new QFormLayout(displayGroupBox);
        formLayout_3->setObjectName("formLayout_3");
        label_4 = new QLabel(displayGroupBox);
        label_4->setObjectName("label_4");

        formLayout_3->setWidget(0, QFormLayout::ItemRole::LabelRole, label_4);

        displayWidthSpinBox = new QSpinBox(displayGroupBox);
        displayWidthSpinBox->setObjectName("displayWidthSpinBox");
        displayWidthSpinBox->setMinimum(640);
        displayWidthSpinBox->setMaximum(2560);
        displayWidthSpinBox->setValue(800);

        formLayout_3->setWidget(0, QFormLayout::ItemRole::FieldRole, displayWidthSpinBox);

        label_5 = new QLabel(displayGroupBox);
        label_5->setObjectName("label_5");

        formLayout_3->setWidget(1, QFormLayout::ItemRole::LabelRole, label_5);

        displayHeightSpinBox = new QSpinBox(displayGroupBox);
        displayHeightSpinBox->setObjectName("displayHeightSpinBox");
        displayHeightSpinBox->setMinimum(480);
        displayHeightSpinBox->setMaximum(1440);
        displayHeightSpinBox->setValue(600);

        formLayout_3->setWidget(1, QFormLayout::ItemRole::FieldRole, displayHeightSpinBox);


        formLayout->setWidget(2, QFormLayout::ItemRole::SpanningRole, displayGroupBox);

        tabWidget->addTab(tabGeneral, QString());
        tabStorage = new QWidget();
        tabStorage->setObjectName("tabStorage");
        verticalLayout_2 = new QVBoxLayout(tabStorage);
        verticalLayout_2->setObjectName("verticalLayout_2");
        bootGroupBox = new QGroupBox(tabStorage);
        bootGroupBox->setObjectName("bootGroupBox");
        formLayout_4 = new QFormLayout(bootGroupBox);
        formLayout_4->setObjectName("formLayout_4");
        label_6 = new QLabel(bootGroupBox);
        label_6->setObjectName("label_6");

        formLayout_4->setWidget(0, QFormLayout::ItemRole::LabelRole, label_6);

        horizontalLayout_2 = new QHBoxLayout();
        horizontalLayout_2->setObjectName("horizontalLayout_2");
        bootCdEdit = new QLineEdit(bootGroupBox);
        bootCdEdit->setObjectName("bootCdEdit");

        horizontalLayout_2->addWidget(bootCdEdit);

        bootCdBrowseButton = new QPushButton(bootGroupBox);
        bootCdBrowseButton->setObjectName("bootCdBrowseButton");

        horizontalLayout_2->addWidget(bootCdBrowseButton);


        formLayout_4->setLayout(0, QFormLayout::ItemRole::FieldRole, horizontalLayout_2);

        label_7 = new QLabel(bootGroupBox);
        label_7->setObjectName("label_7");

        formLayout_4->setWidget(1, QFormLayout::ItemRole::LabelRole, label_7);

        horizontalLayout_3 = new QHBoxLayout();
        horizontalLayout_3->setObjectName("horizontalLayout_3");
        bootDiskEdit = new QLineEdit(bootGroupBox);
        bootDiskEdit->setObjectName("bootDiskEdit");

        horizontalLayout_3->addWidget(bootDiskEdit);

        bootDiskBrowseButton = new QPushButton(bootGroupBox);
        bootDiskBrowseButton->setObjectName("bootDiskBrowseButton");

        horizontalLayout_3->addWidget(bootDiskBrowseButton);


        formLayout_4->setLayout(1, QFormLayout::ItemRole::FieldRole, horizontalLayout_3);


        verticalLayout_2->addWidget(bootGroupBox);

        label_8 = new QLabel(tabStorage);
        label_8->setObjectName("label_8");

        verticalLayout_2->addWidget(label_8);

        verticalSpacer = new QSpacerItem(20, 40, QSizePolicy::Policy::Minimum, QSizePolicy::Policy::Expanding);

        verticalLayout_2->addItem(verticalSpacer);

        tabWidget->addTab(tabStorage, QString());
        tabNetwork = new QWidget();
        tabNetwork->setObjectName("tabNetwork");
        verticalLayout_3 = new QVBoxLayout(tabNetwork);
        verticalLayout_3->setObjectName("verticalLayout_3");
        networkEnabledCheckBox = new QCheckBox(tabNetwork);
        networkEnabledCheckBox->setObjectName("networkEnabledCheckBox");

        verticalLayout_3->addWidget(networkEnabledCheckBox);

        networkSettingsGroupBox = new QGroupBox(tabNetwork);
        networkSettingsGroupBox->setObjectName("networkSettingsGroupBox");
        networkSettingsGroupBox->setEnabled(false);
        formLayout_5 = new QFormLayout(networkSettingsGroupBox);
        formLayout_5->setObjectName("formLayout_5");
        label_9 = new QLabel(networkSettingsGroupBox);
        label_9->setObjectName("label_9");

        formLayout_5->setWidget(0, QFormLayout::ItemRole::LabelRole, label_9);

        networkTypeCombo = new QComboBox(networkSettingsGroupBox);
        networkTypeCombo->addItem(QString());
        networkTypeCombo->addItem(QString());
        networkTypeCombo->setObjectName("networkTypeCombo");

        formLayout_5->setWidget(0, QFormLayout::ItemRole::FieldRole, networkTypeCombo);


        verticalLayout_3->addWidget(networkSettingsGroupBox);

        verticalSpacer_2 = new QSpacerItem(20, 40, QSizePolicy::Policy::Minimum, QSizePolicy::Policy::Expanding);

        verticalLayout_3->addItem(verticalSpacer_2);

        tabWidget->addTab(tabNetwork, QString());

        verticalLayout->addWidget(tabWidget);


        retranslateUi(ConfigEditor);
        QObject::connect(networkEnabledCheckBox, &QCheckBox::toggled, networkSettingsGroupBox, &QGroupBox::setEnabled);

        tabWidget->setCurrentIndex(0);


        QMetaObject::connectSlotsByName(ConfigEditor);
    } // setupUi

    void retranslateUi(QWidget *ConfigEditor)
    {
        cpuGroupBox->setTitle(QCoreApplication::translate("ConfigEditor", "CPU && Memory", nullptr));
        label->setText(QCoreApplication::translate("ConfigEditor", "CPU Model:", nullptr));
        cpuModelCombo->setItemText(0, QCoreApplication::translate("ConfigEditor", "G3 (740)", nullptr));
        cpuModelCombo->setItemText(1, QCoreApplication::translate("ConfigEditor", "G3 (750)", nullptr));
        cpuModelCombo->setItemText(2, QCoreApplication::translate("ConfigEditor", "G4 (7400)", nullptr));
        cpuModelCombo->setItemText(3, QCoreApplication::translate("ConfigEditor", "G4 (7450)", nullptr));

        label_2->setText(QCoreApplication::translate("ConfigEditor", "Clock Speed (MHz):", nullptr));
        label_3->setText(QCoreApplication::translate("ConfigEditor", "RAM Size (MB):", nullptr));
        romGroupBox->setTitle(QCoreApplication::translate("ConfigEditor", "ROM File", nullptr));
        romPathEdit->setPlaceholderText(QCoreApplication::translate("ConfigEditor", "Select ROM file...", nullptr));
        romBrowseButton->setText(QCoreApplication::translate("ConfigEditor", "Browse...", nullptr));
        displayGroupBox->setTitle(QCoreApplication::translate("ConfigEditor", "Display", nullptr));
        label_4->setText(QCoreApplication::translate("ConfigEditor", "Width:", nullptr));
        label_5->setText(QCoreApplication::translate("ConfigEditor", "Height:", nullptr));
        tabWidget->setTabText(tabWidget->indexOf(tabGeneral), QCoreApplication::translate("ConfigEditor", "General", nullptr));
        bootGroupBox->setTitle(QCoreApplication::translate("ConfigEditor", "Boot Devices", nullptr));
        label_6->setText(QCoreApplication::translate("ConfigEditor", "Boot CD/DVD:", nullptr));
        bootCdEdit->setPlaceholderText(QCoreApplication::translate("ConfigEditor", "Optional ISO image...", nullptr));
        bootCdBrowseButton->setText(QCoreApplication::translate("ConfigEditor", "Browse...", nullptr));
        label_7->setText(QCoreApplication::translate("ConfigEditor", "Boot Disk:", nullptr));
        bootDiskEdit->setPlaceholderText(QCoreApplication::translate("ConfigEditor", "Optional disk image...", nullptr));
        bootDiskBrowseButton->setText(QCoreApplication::translate("ConfigEditor", "Browse...", nullptr));
        label_8->setText(QCoreApplication::translate("ConfigEditor", "Storage device management coming soon...", nullptr));
        tabWidget->setTabText(tabWidget->indexOf(tabStorage), QCoreApplication::translate("ConfigEditor", "Storage", nullptr));
        networkEnabledCheckBox->setText(QCoreApplication::translate("ConfigEditor", "Enable Networking", nullptr));
        networkSettingsGroupBox->setTitle(QCoreApplication::translate("ConfigEditor", "Network Settings", nullptr));
        label_9->setText(QCoreApplication::translate("ConfigEditor", "Type:", nullptr));
        networkTypeCombo->setItemText(0, QCoreApplication::translate("ConfigEditor", "User-mode (SLIRP)", nullptr));
        networkTypeCombo->setItemText(1, QCoreApplication::translate("ConfigEditor", "TAP Device", nullptr));

        tabWidget->setTabText(tabWidget->indexOf(tabNetwork), QCoreApplication::translate("ConfigEditor", "Network", nullptr));
        (void)ConfigEditor;
    } // retranslateUi

};

namespace Ui {
    class ConfigEditor: public Ui_ConfigEditor {};
} // namespace Ui

QT_END_NAMESPACE

#endif // UI_CONFIGEDITOR_H
