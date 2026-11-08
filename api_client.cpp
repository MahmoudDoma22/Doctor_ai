#include "api_client.h"
#include <QNetworkRequest>
#include <QNetworkReply>
#include <QJsonDocument>
#include <QJsonArray>
#include <QFile>
#include <QUrl>

ApiClient::ApiClient(QObject *parent) : QObject(parent) {}
void ApiClient::setBaseUrl(const QString &url) { baseUrl_ = url.trimmed(); }

void ApiClient::checkHealth() {
    QNetworkRequest req(QUrl(baseUrl_ + "/api/health"));
    req.setTransferTimeout(5000);
    auto *reply = network_.get(req);
    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        const auto body = reply->readAll();
        const auto health = QJsonDocument::fromJson(body).object();
        const bool ok = reply->error() == QNetworkReply::NoError && health.value("ok").toBool(false);
        const QString message = reply->error() == QNetworkReply::NoError
            ? (ok ? "Local AI services are ready." : health.value("error").toString("Ollama is not ready or no local model is installed."))
            : reply->errorString();
        emit healthResult(ok, message);
        reply->deleteLater();
    });
}

void ApiClient::refreshModels() {
    QNetworkRequest req(QUrl(baseUrl_ + "/api/models"));
    req.setTransferTimeout(10000);
    auto *reply = network_.get(req);
    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        const auto body = reply->readAll();
        if (reply->error() != QNetworkReply::NoError) {
            emit requestError("Could not list local models: " + reply->errorString());
        } else {
            const auto obj = QJsonDocument::fromJson(body).object();
            QStringList names;
            for (const auto &v : obj.value("models").toArray()) names << v.toString();
            emit modelsResult(names, obj.value("activeModel").toString());
        }
        reply->deleteLater();
    });
}

void ApiClient::selectModel(const QString &model) {
    QJsonObject payload; payload["model"] = model;
    QNetworkRequest req(QUrl(baseUrl_ + "/api/models/select"));
    req.setHeader(QNetworkRequest::ContentTypeHeader, "application/json");
    auto *reply = network_.post(req, QJsonDocument(payload).toJson(QJsonDocument::Compact));
    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        const auto body = reply->readAll();
        if (reply->error() != QNetworkReply::NoError) emit modelSelected(false, reply->errorString() + "\n" + QString::fromUtf8(body));
        else emit modelSelected(true, QJsonDocument::fromJson(body).object().value("activeModel").toString());
        reply->deleteLater();
    });
}

void ApiClient::sendMessage(const QString &message, const QStringList &imagePaths, const QStringList &videoPaths, const QString &module) {
    QJsonObject payload;
    payload["message"] = message;
    payload["module"] = module;
    QJsonArray images;
    for (const QString &path : imagePaths) {
        QFile file(path);
        if (!file.open(QIODevice::ReadOnly)) continue;
        images.append(QString::fromLatin1(file.readAll().toBase64()));
    }
    payload["images"] = images;
    QJsonArray videos;
    for (const QString &path : videoPaths) {
        QFile file(path);
        if (!file.open(QIODevice::ReadOnly) || file.size() > 100LL * 1024 * 1024) continue;
        videos.append(QString::fromLatin1(file.readAll().toBase64()));
    }
    payload["videos"] = videos;
    QNetworkRequest req(QUrl(baseUrl_ + "/api/chat"));
    req.setHeader(QNetworkRequest::ContentTypeHeader, "application/json");
    req.setTransferTimeout(180000);
    auto *reply = network_.post(req, QJsonDocument(payload).toJson(QJsonDocument::Compact));
    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        const auto body = reply->readAll();
        if (reply->error() != QNetworkReply::NoError) emit requestError(reply->errorString() + "\n" + QString::fromUtf8(body));
        else emit chatResult(QJsonDocument::fromJson(body).object().value("answer").toString("No answer returned."));
        reply->deleteLater();
    });
}

void ApiClient::saveSkill(const QString &module, const QString &name, const QString &content) {
    QJsonObject payload; payload["module"] = module; payload["name"] = name; payload["content"] = content;
    QNetworkRequest req(QUrl(baseUrl_ + "/api/skills"));
    req.setHeader(QNetworkRequest::ContentTypeHeader, "application/json");
    auto *reply = network_.post(req, QJsonDocument(payload).toJson(QJsonDocument::Compact));
    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        const bool ok = reply->error() == QNetworkReply::NoError;
        emit skillSaved(ok, ok ? "Skill saved." : reply->errorString() + "\n" + QString::fromUtf8(reply->readAll()));
        reply->deleteLater();
    });
}
