#include "api_client.h"
#include <QNetworkRequest>
#include <QNetworkReply>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
ApiClient::ApiClient(QObject *parent):QObject(parent){}
void ApiClient::checkHealth(){
    QNetworkRequest req(QUrl(baseUrl_+"/api/health")); req.setTransferTimeout(3000);
    auto *r=network_.get(req);
    connect(r,&QNetworkReply::finished,this,[this,r]{
        auto doc=QJsonDocument::fromJson(r->readAll()); bool ok=r->error()==QNetworkReply::NoError && doc.object().value("ok").toBool();
        emit healthChanged(ok, ok?"Local services are ready":doc.object().value("message").toString(r->errorString())); r->deleteLater();
    });
}
void ApiClient::fetchModels(){
    auto *r=network_.get(QNetworkRequest(QUrl(baseUrl_+"/api/models")));
    connect(r,&QNetworkReply::finished,this,[this,r]{
        auto doc=QJsonDocument::fromJson(r->readAll()); QStringList names;
        for(const auto &v:doc.object().value("models").toArray()) names<<v.toString();
        emit modelsReceived(names,doc.object().value("active").toString()); if(r->error()!=QNetworkReply::NoError) emit failed(r->errorString()); r->deleteLater();
    });
}
void ApiClient::sendMessage(const QString &message,const QString &model){
    QJsonObject body{{"message",message},{"model",model}};
    QNetworkRequest req(QUrl(baseUrl_+"/api/chat")); req.setHeader(QNetworkRequest::ContentTypeHeader,"application/json"); req.setTransferTimeout(180000);
    auto *r=network_.post(req,QJsonDocument(body).toJson(QJsonDocument::Compact));
    connect(r,&QNetworkReply::finished,this,[this,r]{
        auto doc=QJsonDocument::fromJson(r->readAll()); if(r->error()!=QNetworkReply::NoError) emit failed(doc.object().value("message").toString(r->errorString()));
        else emit answerReceived(doc.object().value("answer").toString("Empty response")); r->deleteLater();
    });
}
