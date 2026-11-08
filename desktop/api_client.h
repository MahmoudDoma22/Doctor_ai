#pragma once
#include <QObject>
#include <QNetworkAccessManager>
#include <QStringList>

class ApiClient : public QObject {
    Q_OBJECT
public:
    explicit ApiClient(QObject *parent=nullptr);
    void checkHealth();
    void fetchModels();
    void sendMessage(const QString &message, const QString &model, const QStringList &attachments = {}, const QString &performanceMode = "balanced");
    void fetchSettings();
    void saveOllamaUrl(const QString &url);
signals:
    void healthChanged(bool ok, const QString &message);
    void modelsReceived(const QStringList &models, const QString &active);
    void answerReceived(const QString &answer);
    void settingsReceived(const QString &ollamaUrl);
    void settingsSaved(bool ok, const QString &message);
    void failed(const QString &message);
private:
    QNetworkAccessManager network_;
    QString baseUrl_ = "http://127.0.0.1:8787";
};