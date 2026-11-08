#include "api_client.h"
#include <QNetworkRequest>
#include <QNetworkReply>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QUrl>
#include <QFile>
#include <QFileInfo>
#include <QMimeDatabase>
ApiClient::ApiClient(QObject *parent):QObject(parent){}
void ApiClient::checkHealth(){
 QNetworkRequest req(QUrl(baseUrl_+"/api/health"));req.setTransferTimeout(5000);auto *r=network_.get(req);
 connect(r,&QNetworkReply::finished,this,[this,r]{auto doc=QJsonDocument::fromJson(r->readAll());bool ready=r->error()==QNetworkReply::NoError&&doc.object().value("ready").toBool();emit healthChanged(ready,doc.object().value("message").toString(r->errorString()));r->deleteLater();});
}
void ApiClient::fetchModels(){
 QNetworkRequest req(QUrl(baseUrl_+"/api/models"));req.setTransferTimeout(15000);auto *r=network_.get(req);
 connect(r,&QNetworkReply::finished,this,[this,r]{auto doc=QJsonDocument::fromJson(r->readAll());QStringList names;for(const auto &v:doc.object().value("models").toArray())names<<v.toString();emit modelsReceived(names,doc.object().value("active").toString());if(r->error()!=QNetworkReply::NoError)emit failed("Local model check failed: "+r->errorString());r->deleteLater();});
}
void ApiClient::sendMessage(const QString &message,const QString &model,const QStringList &attachments,const QString &mode){
 QString prompt=message;QJsonArray images;QMimeDatabase db;
 for(const QString &path:attachments){
  QFileInfo info(path);QString suffix=info.suffix().toLower();QFile file(path);
  if(!file.open(QIODevice::ReadOnly)){emit failed("Cannot read attachment: "+info.fileName());return;}
  if(info.size()>15*1024*1024){emit failed("Attachment is larger than 15 MB: "+info.fileName());return;}
  QByteArray data=file.readAll();QString mime=db.mimeTypeForFile(info).name();
  if(mime.startsWith("image/"))images.append(QString::fromLatin1(data.toBase64()));
  else if(suffix=="txt"||suffix=="md"||suffix=="csv"||suffix=="log"){if(data.size()>2*1024*1024){emit failed("Text attachment is larger than 2 MB: "+info.fileName());return;}prompt+="\n\n[Attached file: "+info.fileName()+"]\n"+QString::fromUtf8(data);}
  else if(suffix=="mp4"||suffix=="mov"||suffix=="avi"||suffix=="mkv"||suffix=="webm"){emit failed("Video attachment selected: "+info.fileName()+". Video frame extraction is not enabled in this build yet.");return;}
  else{emit failed("Unsupported file type: "+info.fileName()+". Currently supported: images, TXT, MD, CSV and LOG.");return;}
 }
 QJsonObject body{{"message",prompt},{"model",model},{"images",images},{"performanceMode",mode}};
 QNetworkRequest req(QUrl(baseUrl_+"/api/chat"));req.setHeader(QNetworkRequest::ContentTypeHeader,"application/json");req.setTransferTimeout(600000);
 auto *r=network_.post(req,QJsonDocument(body).toJson(QJsonDocument::Compact));
 connect(r,&QNetworkReply::finished,this,[this,r]{auto doc=QJsonDocument::fromJson(r->readAll());if(r->error()!=QNetworkReply::NoError)emit failed(doc.object().value("message").toString(r->errorString()));else if(doc.object().contains("message"))emit failed(doc.object().value("message").toString());else emit answerReceived(doc.object().value("answer").toString("Empty response"));r->deleteLater();});
}