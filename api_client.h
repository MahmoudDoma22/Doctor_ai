#pragma once
#include <QObject>
#include <QNetworkAccessManager>
#include <QJsonObject>
#include <QStringList>

class ApiClient : public QObject {
    Q_OBJECT
public:
    explicit ApiClient(QObject *parent = nullptr);
    void setBaseUrl(const QString &url);
    void checkHealth();
    void refreshModels();
    void selectModel(const QString &model);
    void sendMessage(const QString &message, const QStringList &imagePaths, const QStringList &videoPaths, const QString &module);
    void saveSkill(const QString &module, const QString &name, const QString &content);
signals:
    void healthResult(bool ok, const QString &message);
    void chatResult(const QString &answer);
    void requestError(const QString &message);
    void skillSaved(bool ok, const QString &message);
    void modelsResult(const QStringList &models, const QString &activeModel);
    void modelSelected(bool ok, const QString &modelOrError);
private:
    QNetworkAccessManager network_;
    QString baseUrl_ = "http://127.0.0.1:8787";
};
