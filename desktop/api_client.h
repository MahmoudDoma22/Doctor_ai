#pragma once
#include <QObject>
#include <QNetworkAccessManager>
#include <QStringList>
class ApiClient:public QObject{
 Q_OBJECT
public:
 explicit ApiClient(QObject *parent=nullptr);
 void checkHealth();
 void fetchModels();
 void sendMessage(const QString &message,const QString &model,const QStringList &attachments={},const QString &performanceMode="balanced");
signals:
 void healthChanged(bool ready,const QString &message);
 void modelsReceived(const QStringList &models,const QString &active);
 void answerReceived(const QString &answer);
 void failed(const QString &message);
private:
 QNetworkAccessManager network_;
 QString baseUrl_="http://127.0.0.1:8787";
};