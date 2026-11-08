#pragma once
#include <QMainWindow>
#include <QStackedWidget>
#include <QTextEdit>
#include <QLineEdit>
#include <QLabel>
#include <QPushButton>
#include <QStringList>
#include "api_client.h"

class MainWindow : public QMainWindow {
    Q_OBJECT
public:
    explicit MainWindow(QWidget *parent = nullptr);
private:
    QStackedWidget *pages_;
    ApiClient api_;
    QLabel *status_;
    QTextEdit *chatView_;
    QLineEdit *composer_;
    QStringList attachments_;
    QStringList videos_;
    QWidget *makeWelcome();
    QWidget *makeConnect();
    QWidget *makeChat();
    void appendMessage(const QString &who, const QString &text);
    void chooseFiles();
    void send();
    void applyTheme();
};
