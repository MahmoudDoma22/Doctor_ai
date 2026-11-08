#pragma once
#include <QMainWindow>
#include <QComboBox>
#include <QLabel>
#include <QLineEdit>
#include <QTextEdit>
#include <QPushButton>
#include <QStringList>
#include "api_client.h"

class MainWindow : public QMainWindow {
    Q_OBJECT
public:
    explicit MainWindow(QWidget *parent=nullptr);
private:
    ApiClient api_;
    QLabel *status_;
    QLabel *modelCount_;
    QComboBox *models_;
    QComboBox *performance_;
    QTextEdit *conversation_;
    QLineEdit *input_;
    QPushButton *send_;
    QLabel *attachmentLabel_;
    QStringList attachments_;
    void append(const QString &speaker,const QString &text);
    void setBusy(bool busy);
    void openSettings();
    void searchLocalModels();
    void addAttachments();
    void clearAttachments();
};