/********************************************************************************
** Form generated from reading UI file 'mainwindow.ui'
**
** Created by: Qt User Interface Compiler version 6.11.1
**
** WARNING! All changes made in this file will be lost when recompiling UI file!
********************************************************************************/

#ifndef UI_MAINWINDOW_H
#define UI_MAINWINDOW_H

#include <QtCore/QVariant>
#include <QtGui/QAction>
#include <QtWidgets/QApplication>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QMenu>
#include <QtWidgets/QMenuBar>
#include <QtWidgets/QStatusBar>
#include <QtWidgets/QToolBar>
#include <QtWidgets/QWidget>

QT_BEGIN_NAMESPACE

class Ui_MainWindow
{
public:
    QAction *actionNew;
    QAction *actionOpen;
    QAction *actionSave;
    QAction *actionSaveAs;
    QAction *actionExit;
    QAction *actionLaunch;
    QAction *actionStop;
    QAction *actionDebugger;
    QAction *actionHfsManager;
    QAction *actionNetworkMonitor;
    QAction *actionDiskManager;
    QAction *actionAbout;
    QAction *actionAboutQt;
    QWidget *centralwidget;
    QMenuBar *menubar;
    QMenu *menuFile;
    QMenu *menuEmulator;
    QMenu *menuTools;
    QMenu *menuHelp;
    QStatusBar *statusbar;
    QToolBar *toolBar;

    void setupUi(QMainWindow *MainWindow)
    {
        if (MainWindow->objectName().isEmpty())
            MainWindow->setObjectName("MainWindow");
        MainWindow->resize(900, 700);
        actionNew = new QAction(MainWindow);
        actionNew->setObjectName("actionNew");
        QIcon icon(QIcon::fromTheme(QString::fromUtf8("document-new")));
        actionNew->setIcon(icon);
        actionOpen = new QAction(MainWindow);
        actionOpen->setObjectName("actionOpen");
        QIcon icon1(QIcon::fromTheme(QString::fromUtf8("document-open")));
        actionOpen->setIcon(icon1);
        actionSave = new QAction(MainWindow);
        actionSave->setObjectName("actionSave");
        QIcon icon2(QIcon::fromTheme(QString::fromUtf8("document-save")));
        actionSave->setIcon(icon2);
        actionSaveAs = new QAction(MainWindow);
        actionSaveAs->setObjectName("actionSaveAs");
        QIcon icon3(QIcon::fromTheme(QString::fromUtf8("document-save-as")));
        actionSaveAs->setIcon(icon3);
        actionExit = new QAction(MainWindow);
        actionExit->setObjectName("actionExit");
        QIcon icon4(QIcon::fromTheme(QString::fromUtf8("application-exit")));
        actionExit->setIcon(icon4);
        actionLaunch = new QAction(MainWindow);
        actionLaunch->setObjectName("actionLaunch");
        QIcon icon5(QIcon::fromTheme(QString::fromUtf8("media-playback-start")));
        actionLaunch->setIcon(icon5);
        actionStop = new QAction(MainWindow);
        actionStop->setObjectName("actionStop");
        actionStop->setEnabled(false);
        QIcon icon6(QIcon::fromTheme(QString::fromUtf8("media-playback-stop")));
        actionStop->setIcon(icon6);
        actionDebugger = new QAction(MainWindow);
        actionDebugger->setObjectName("actionDebugger");
        QIcon icon7(QIcon::fromTheme(QString::fromUtf8("utilities-system-monitor")));
        actionDebugger->setIcon(icon7);
        actionHfsManager = new QAction(MainWindow);
        actionHfsManager->setObjectName("actionHfsManager");
        QIcon icon8(QIcon::fromTheme(QString::fromUtf8("folder-open")));
        actionHfsManager->setIcon(icon8);
        actionNetworkMonitor = new QAction(MainWindow);
        actionNetworkMonitor->setObjectName("actionNetworkMonitor");
        QIcon icon9(QIcon::fromTheme(QString::fromUtf8("network-wired")));
        actionNetworkMonitor->setIcon(icon9);
        actionDiskManager = new QAction(MainWindow);
        actionDiskManager->setObjectName("actionDiskManager");
        QIcon icon10(QIcon::fromTheme(QString::fromUtf8("drive-harddisk")));
        actionDiskManager->setIcon(icon10);
        actionAbout = new QAction(MainWindow);
        actionAbout->setObjectName("actionAbout");
        QIcon icon11(QIcon::fromTheme(QString::fromUtf8("help-about")));
        actionAbout->setIcon(icon11);
        actionAboutQt = new QAction(MainWindow);
        actionAboutQt->setObjectName("actionAboutQt");
        centralwidget = new QWidget(MainWindow);
        centralwidget->setObjectName("centralwidget");
        MainWindow->setCentralWidget(centralwidget);
        menubar = new QMenuBar(MainWindow);
        menubar->setObjectName("menubar");
        menubar->setGeometry(QRect(0, 0, 900, 22));
        menuFile = new QMenu(menubar);
        menuFile->setObjectName("menuFile");
        menuEmulator = new QMenu(menubar);
        menuEmulator->setObjectName("menuEmulator");
        menuTools = new QMenu(menubar);
        menuTools->setObjectName("menuTools");
        menuHelp = new QMenu(menubar);
        menuHelp->setObjectName("menuHelp");
        MainWindow->setMenuBar(menubar);
        statusbar = new QStatusBar(MainWindow);
        statusbar->setObjectName("statusbar");
        MainWindow->setStatusBar(statusbar);
        toolBar = new QToolBar(MainWindow);
        toolBar->setObjectName("toolBar");
        MainWindow->addToolBar(Qt::ToolBarArea::TopToolBarArea, toolBar);

        menubar->addAction(menuFile->menuAction());
        menubar->addAction(menuEmulator->menuAction());
        menubar->addAction(menuTools->menuAction());
        menubar->addAction(menuHelp->menuAction());
        menuFile->addAction(actionNew);
        menuFile->addAction(actionOpen);
        menuFile->addAction(actionSave);
        menuFile->addAction(actionSaveAs);
        menuFile->addSeparator();
        menuFile->addAction(actionExit);
        menuEmulator->addAction(actionLaunch);
        menuEmulator->addAction(actionStop);
        menuTools->addAction(actionDebugger);
        menuTools->addAction(actionHfsManager);
        menuTools->addAction(actionNetworkMonitor);
        menuTools->addAction(actionDiskManager);
        menuHelp->addAction(actionAbout);
        menuHelp->addAction(actionAboutQt);
        toolBar->addAction(actionNew);
        toolBar->addAction(actionOpen);
        toolBar->addAction(actionSave);
        toolBar->addSeparator();
        toolBar->addAction(actionLaunch);
        toolBar->addAction(actionStop);

        retranslateUi(MainWindow);

        QMetaObject::connectSlotsByName(MainWindow);
    } // setupUi

    void retranslateUi(QMainWindow *MainWindow)
    {
        MainWindow->setWindowTitle(QCoreApplication::translate("MainWindow", "NewtonEmu Frontend", nullptr));
        actionNew->setText(QCoreApplication::translate("MainWindow", "&New", nullptr));
#if QT_CONFIG(shortcut)
        actionNew->setShortcut(QCoreApplication::translate("MainWindow", "Ctrl+N", nullptr));
#endif // QT_CONFIG(shortcut)
        actionOpen->setText(QCoreApplication::translate("MainWindow", "&Open...", nullptr));
#if QT_CONFIG(shortcut)
        actionOpen->setShortcut(QCoreApplication::translate("MainWindow", "Ctrl+O", nullptr));
#endif // QT_CONFIG(shortcut)
        actionSave->setText(QCoreApplication::translate("MainWindow", "&Save", nullptr));
#if QT_CONFIG(shortcut)
        actionSave->setShortcut(QCoreApplication::translate("MainWindow", "Ctrl+S", nullptr));
#endif // QT_CONFIG(shortcut)
        actionSaveAs->setText(QCoreApplication::translate("MainWindow", "Save &As...", nullptr));
#if QT_CONFIG(shortcut)
        actionSaveAs->setShortcut(QCoreApplication::translate("MainWindow", "Ctrl+Shift+S", nullptr));
#endif // QT_CONFIG(shortcut)
        actionExit->setText(QCoreApplication::translate("MainWindow", "E&xit", nullptr));
#if QT_CONFIG(shortcut)
        actionExit->setShortcut(QCoreApplication::translate("MainWindow", "Ctrl+Q", nullptr));
#endif // QT_CONFIG(shortcut)
        actionLaunch->setText(QCoreApplication::translate("MainWindow", "&Launch Emulator", nullptr));
#if QT_CONFIG(tooltip)
        actionLaunch->setToolTip(QCoreApplication::translate("MainWindow", "Launch the emulator with current configuration", nullptr));
#endif // QT_CONFIG(tooltip)
#if QT_CONFIG(shortcut)
        actionLaunch->setShortcut(QCoreApplication::translate("MainWindow", "F5", nullptr));
#endif // QT_CONFIG(shortcut)
        actionStop->setText(QCoreApplication::translate("MainWindow", "&Stop Emulator", nullptr));
#if QT_CONFIG(tooltip)
        actionStop->setToolTip(QCoreApplication::translate("MainWindow", "Stop the running emulator", nullptr));
#endif // QT_CONFIG(tooltip)
#if QT_CONFIG(shortcut)
        actionStop->setShortcut(QCoreApplication::translate("MainWindow", "Shift+F5", nullptr));
#endif // QT_CONFIG(shortcut)
        actionDebugger->setText(QCoreApplication::translate("MainWindow", "&Debugger", nullptr));
#if QT_CONFIG(tooltip)
        actionDebugger->setToolTip(QCoreApplication::translate("MainWindow", "Open the live debugger", nullptr));
#endif // QT_CONFIG(tooltip)
        actionHfsManager->setText(QCoreApplication::translate("MainWindow", "&HFS+ File Manager", nullptr));
#if QT_CONFIG(tooltip)
        actionHfsManager->setToolTip(QCoreApplication::translate("MainWindow", "Browse and manage HFS+ disk images", nullptr));
#endif // QT_CONFIG(tooltip)
        actionNetworkMonitor->setText(QCoreApplication::translate("MainWindow", "&Network Monitor", nullptr));
#if QT_CONFIG(tooltip)
        actionNetworkMonitor->setToolTip(QCoreApplication::translate("MainWindow", "Monitor network traffic", nullptr));
#endif // QT_CONFIG(tooltip)
        actionDiskManager->setText(QCoreApplication::translate("MainWindow", "Disk &Manager", nullptr));
#if QT_CONFIG(tooltip)
        actionDiskManager->setToolTip(QCoreApplication::translate("MainWindow", "Manage virtual disk images", nullptr));
#endif // QT_CONFIG(tooltip)
        actionAbout->setText(QCoreApplication::translate("MainWindow", "&About", nullptr));
        actionAboutQt->setText(QCoreApplication::translate("MainWindow", "About &Qt", nullptr));
        menuFile->setTitle(QCoreApplication::translate("MainWindow", "&File", nullptr));
        menuEmulator->setTitle(QCoreApplication::translate("MainWindow", "&Emulator", nullptr));
        menuTools->setTitle(QCoreApplication::translate("MainWindow", "&Tools", nullptr));
        menuHelp->setTitle(QCoreApplication::translate("MainWindow", "&Help", nullptr));
        toolBar->setWindowTitle(QCoreApplication::translate("MainWindow", "toolBar", nullptr));
    } // retranslateUi

};

namespace Ui {
    class MainWindow: public Ui_MainWindow {};
} // namespace Ui

QT_END_NAMESPACE

#endif // UI_MAINWINDOW_H
