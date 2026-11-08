#include "api_client.h"
#include <QNetworkRequest>
#include <QNetworkReply>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QUrl>

ApiClient::ApiClient(QObject *parent):QObject(parent){}

void ApiClient::checkHealth(){
    QNetworkRequest req(QUrl(baseUrl_+"/api/health"));
    req.setTransferTimeout(5000);
    auto *r=network_.get(req);
    connect(r,&QNetworkReply::finished,this,[this,r]{
        const auto doc=QJsonDocument::fromJson(r->readAll());
        const bool ok=r->error()==QNetworkReply::NoError && doc.object().value("ok").toBool();
        emit healthChanged(ok,ok?"Ollama connected":doc.object().value("message").toString(r->errorString()));
        r->deleteLater();
    });
}

void ApiClient::fetchModels(){
    QNetworkRequest req(QUrl(baseUrl_+"/api/models"));
    req.setTransferTimeout(15000);
    auto *r=network_.get(req);
    connect(r,&QNetworkReply::finished,this,[this,r]{
        const auto doc=QJsonDocument::fromJson(r->readAll());
        QStringList names;
        for(const auto &v:doc.object().value("models").toArray()) names<<v.toString();
        emit modelsReceived(names,doc.object().value("active").toString());
        if(r->error()!=QNetworkReply::NoError) emit failed("Local model search failed: "+r->errorString());
        r->deleteLater();
    });
}

void ApiClient::sendMessage(const QString &message,const QString &model){
    QJsonObject body{{"message",message},{"model",model}};
    QNetworkRequest req(QUrl(baseUrl_+"/api/chat"));
    req.setHeader(QNetworkRequest::ContentTypeHeader,"application/json");
    req.setTransferTimeout(180000);
    auto *r=network_.post(req,QJsonDocument(body).toJson(QJsonDocument::Compact));
    connect(r,&QNetworkReply::finished,this,[this,r]{
        const auto doc=QJsonDocument::fromJson(r->readAll());
        if(r->error()!=QNetworkReply::NoError) emit failed(doc.object().value("message").toString(r->errorString()));
        else if(doc.object().contains("message")) emit failed(doc.object().value("message").toString());
        else emit answerReceived(doc.object().value("answer").toString("Empty response"));
        r->deleteLater();
    });
}

void ApiClient::fetchSettings(){
    auto *r=network_.get(QNetworkRequest(QUrl(baseUrl_+"/api/settings")));
    connect(r,&QNetworkReply::finished,this,[this,r]{
        const auto doc=QJsonDocument::fromJson(r->readAll());
        if(r->error()==QNetworkReply::NoError) emit settingsReceived(doc.object().value("ollamaUrl").toString());
        else emit failed("Could not load settings: "+r->errorString());
        r->deleteLater();
    });
}

void ApiClient::saveOllamaUrl(const QString &url){
    QJsonObject body{{"ollamaUrl",url}};
    QNetworkRequest req(QUrl(baseUrl_+"/api/settings"));
    req.setHeader(QNetworkRequest::ContentTypeHeader,"application/json");
    auto *r=network_.post(req,QJsonDocument(body).toJson(QJsonDocument::Compact));
    connect(r,&QNetworkReply::finished,this,[this,r]{
        const auto doc=QJsonDocument::fromJson(r->readAll());
        const bool ok=r->error()==QNetworkReply::NoError && doc.object().value("ok").toBool();
        emit settingsSaved(ok,doc.object().value("message").toString(r->errorString()));
        r->deleteLater();
    });
}
