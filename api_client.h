#pragma once
#include <QObject>
#include <QNetworkAccessManager>
#include <QJsonObject>

class ApiClient : public QObject {
    Q_OBJECT
public:
    explicit ApiClient(QObject *parent = nullptr);
    void setBaseUrl(const QString &url);
    void checkHealth();
    void sendMessage(const QString &message, const QStringList &imagePaths, const QStringList &videoPaths);
    void saveSkill(const QString &name, const QString &content);
signals:
    void healthResult(bool ok, const QString &message);
    void chatResult(const QString &answer);
    void requestError(const QString &message);
    void skillSaved(bool ok, const QString &message);
private:
    QNetworkAccessManager network_;
    QString baseUrl_ = "http://127.0.0.1:8787";
};
