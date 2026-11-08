#include "mainwindow.h"
#include <QHBoxLayout>
#include <QVBoxLayout>
#include <QFrame>
#include <QMessageBox>
MainWindow::MainWindow(QWidget *parent):QMainWindow(parent){
 setWindowTitle("Doctor AI"); resize(1180,780); setMinimumSize(900,620);
 auto *root=new QWidget; auto *outer=new QHBoxLayout(root); outer->setContentsMargins(0,0,0,0); outer->setSpacing(0);
 auto *side=new QFrame; side->setObjectName("sidebar"); side->setFixedWidth(235); auto *sl=new QVBoxLayout(side); sl->setContentsMargins(22,26,22,22);
 auto *brand=new QLabel("✚  DOCTOR AI"); brand->setStyleSheet("font-size:21px;font-weight:800;color:#55c2ff"); sl->addWidget(brand);
 auto *tag=new QLabel("LOCAL INTELLIGENCE WORKSPACE"); tag->setStyleSheet("font-size:9px;color:#8296ae;letter-spacing:1px"); sl->addWidget(tag); sl->addSpacing(30);
 auto *section=new QLabel("WORKSPACE"); section->setStyleSheet("color:#8195ad;font-size:10px;font-weight:700"); sl->addWidget(section);
 for(const QString &s:{"General assistant","Medical assistant","Image studio","Video studio","Audio studio"}){auto *b=new QPushButton("   "+s);b->setCursor(Qt::PointingHandCursor);sl->addWidget(b);b->setEnabled(s=="General assistant"||s=="Medical assistant");}
 sl->addStretch(); status_=new QLabel("● Checking local services…");status_->setWordWrap(true);sl->addWidget(status_);
 auto *main=new QWidget;auto *ml=new QVBoxLayout(main);ml->setContentsMargins(30,25,30,24);
 auto *top=new QHBoxLayout;auto *heading=new QLabel("Your assistant");heading->setStyleSheet("font-size:25px;font-weight:750");top->addWidget(heading);top->addStretch();
 models_=new QComboBox;models_->setMinimumWidth(210);models_->addItem("Discovering local models…");top->addWidget(new QLabel("MODEL"));top->addWidget(models_);auto *refresh=new QPushButton("↻");refresh->setToolTip("Refresh installed models");top->addWidget(refresh);ml->addLayout(top);
 auto *sub=new QLabel("Private by design · Runs on this computer");sub->setStyleSheet("color:#8fa4bd");ml->addWidget(sub);
 conversation_=new QTextEdit;conversation_->setReadOnly(true);conversation_->setPlaceholderText("Start a conversation. Your prompts are sent to your local model.");ml->addWidget(conversation_,1);
 auto *hint=new QLabel("LOCAL MODEL CHAT  ·  Never use AI output as the sole basis for clinical decisions.");hint->setStyleSheet("color:#8296ae;font-size:10px");ml->addWidget(hint);
 auto *row=new QHBoxLayout;input_=new QLineEdit;input_->setPlaceholderText("Ask Doctor AI…");send_=new QPushButton("Send  ➜");send_->setObjectName("primary");row->addWidget(input_,1);row->addWidget(send_);ml->addLayout(row);
 outer->addWidget(side);outer->addWidget(main,1);setCentralWidget(root);
 setStyleSheet(R"(
 QMainWindow,QWidget{background:#0b1422;color:#edf4ff;font-family:"Segoe UI";font-size:13px}
 QFrame#sidebar{background:#101d2e;border-right:1px solid #23364c}
 QPushButton{background:#182a3f;color:#dce9fa;border:1px solid #2a4059;border-radius:9px;padding:11px;text-align:left}
 QPushButton:hover{background:#233d58} QPushButton#primary{background:#168bd2;border:0;color:white;font-weight:700;text-align:center;padding:12px 22px}
 QLineEdit,QTextEdit,QComboBox{background:#101e30;border:1px solid #293e57;border-radius:10px;padding:12px;color:#edf4ff}
 )");
 connect(refresh,&QPushButton::clicked,&api_,&ApiClient::fetchModels);
 connect(send_,&QPushButton::clicked,this,[this]{auto m=input_->text().trimmed();if(m.isEmpty())return;append("You",m);input_->clear();setBusy(true);api_.sendMessage(m,models_->currentText());});
 connect(input_,&QLineEdit::returnPressed,send_,&QPushButton::click);
 connect(&api_,&ApiClient::healthChanged,this,[this](bool ok,const QString &m){status_->setText(ok?"● Local services online":"● "+m);status_->setStyleSheet(ok?"color:#49d6a2":"color:#ff8b8b");});
 connect(&api_,&ApiClient::modelsReceived,this,[this](const QStringList &names,const QString &active){models_->clear();models_->addItems(names);int i=models_->findText(active);if(i>=0)models_->setCurrentIndex(i);if(names.isEmpty())models_->addItem("No local models found");});
 connect(&api_,&ApiClient::answerReceived,this,[this](const QString &a){append("Doctor AI",a);setBusy(false);});
 connect(&api_,&ApiClient::failed,this,[this](const QString &e){append("System",e);setBusy(false);});
 api_.checkHealth();api_.fetchModels();
}
void MainWindow::append(const QString &s,const QString &t){conversation_->append("<b>"+s.toHtmlEscaped()+"</b><br>"+t.toHtmlEscaped().replace("\n","<br>")+"<br>");}
void MainWindow::setBusy(bool b){send_->setEnabled(!b);input_->setEnabled(!b);}
