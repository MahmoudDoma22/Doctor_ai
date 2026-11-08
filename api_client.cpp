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
        const bool ok = reply->error() == QNetworkReply::NoError;
        const QString message = ok ? QString::fromUtf8(body) : reply->errorString();
        emit healthResult(ok, message);
        reply->deleteLater();
    });
}

void ApiClient::sendMessage(const QString &message, const QStringList &imagePaths, const QStringList &videoPaths) {
    QJsonObject payload;
    payload["message"] = message;
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
        if (reply->error() != QNetworkReply::NoError) {
            emit requestError(reply->errorString() + "\n" + QString::fromUtf8(body));
        } else {
            const auto doc = QJsonDocument::fromJson(body);
            emit chatResult(doc.object().value("answer").toString("No answer returned."));
        }
        reply->deleteLater();
    });
}

void ApiClient::saveSkill(const QString &name, const QString &content) {
    QJsonObject payload;
    payload["name"] = name;
    payload["content"] = content;
    QNetworkRequest req(QUrl(baseUrl_ + "/api/skills"));
    req.setHeader(QNetworkRequest::ContentTypeHeader, "application/json");
    auto *reply = network_.post(req, QJsonDocument(payload).toJson(QJsonDocument::Compact));
    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        const bool ok = reply->error() == QNetworkReply::NoError;
        emit skillSaved(ok, ok ? "Skill saved." : reply->errorString());
        reply->deleteLater();
    });
}
