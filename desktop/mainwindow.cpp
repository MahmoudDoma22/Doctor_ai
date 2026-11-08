#include "mainwindow.h"
#include <QHBoxLayout>
#include <QVBoxLayout>
#include <QFrame>
#include <QMessageBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFileDialog>
#include <QFileInfo>
#include <QTimer>
MainWindow::MainWindow(QWidget *parent):QMainWindow(parent){
 setWindowTitle("Doctor AI");resize(1240,820);setMinimumSize(960,650);
 auto *root=new QWidget;auto *outer=new QHBoxLayout(root);outer->setContentsMargins(0,0,0,0);outer->setSpacing(0);
 auto *side=new QFrame;side->setObjectName("sidebar");side->setFixedWidth(245);auto *sl=new QVBoxLayout(side);sl->setContentsMargins(22,26,22,22);
 auto *brand=new QLabel("✚  DOCTOR AI");brand->setStyleSheet("font-size:22px;font-weight:800;color:#55c2ff");sl->addWidget(brand);
 auto *tag=new QLabel("LOCAL INTELLIGENCE WORKSPACE");tag->setStyleSheet("font-size:9px;color:#8296ae;letter-spacing:1px");sl->addWidget(tag);sl->addSpacing(30);
 auto *section=new QLabel("WORKSPACE");section->setStyleSheet("color:#8195ad;font-size:10px;font-weight:700");sl->addWidget(section);
 for(const QString &s:QStringList{"General assistant","Medical assistant","Image studio","Video studio","Audio studio"}){auto *b=new QPushButton("   "+s);b->setCursor(Qt::PointingHandCursor);b->setEnabled(s=="General assistant"||s=="Medical assistant");sl->addWidget(b);}
 sl->addStretch();status_=new QLabel("● Starting offline AI engine…");status_->setWordWrap(true);sl->addWidget(status_);
 modelCount_=new QLabel("Offline model: checking…");modelCount_->setStyleSheet("color:#8296ae;font-size:10px");sl->addWidget(modelCount_);
 auto *settings=new QPushButton("⚙  Offline modules");settings->setObjectName("settingsButton");settings->setCursor(Qt::PointingHandCursor);sl->addWidget(settings);
 auto *main=new QWidget;auto *ml=new QVBoxLayout(main);ml->setContentsMargins(30,25,30,24);ml->setSpacing(14);
 auto *top=new QHBoxLayout;auto *heading=new QLabel("Your assistant");heading->setStyleSheet("font-size:25px;font-weight:750");top->addWidget(heading);top->addStretch();
 models_=new QComboBox;models_->setMinimumWidth(240);models_->addItem("Checking installed model…");top->addWidget(new QLabel("MODEL"));top->addWidget(models_);
 auto *search=new QPushButton("↻  Check model");search->setObjectName("searchButton");search->setCursor(Qt::PointingHandCursor);top->addWidget(search);ml->addLayout(top);
 auto *sub=new QLabel("Fully local inference · No Ollama · No cloud required for offline modules");sub->setStyleSheet("color:#8fa4bd");ml->addWidget(sub);
 conversation_=new QTextEdit;conversation_->setReadOnly(true);conversation_->setPlaceholderText("Welcome to Doctor AI. Install the recommended offline model during setup to begin.");ml->addWidget(conversation_,1);
 auto *hint=new QLabel("Model selection is based on system RAM. CPU inference is supported; GPU acceleration is not included in this build.");hint->setStyleSheet("color:#8296ae;font-size:10px");ml->addWidget(hint);
 auto *attachRow=new QHBoxLayout;auto *attach=new QPushButton("＋ Attach files");attach->setObjectName("searchButton");auto *clear=new QPushButton("Clear attachments");attachmentLabel_=new QLabel("No attachments");attachmentLabel_->setStyleSheet("color:#91a7c0");attachRow->addWidget(attach);attachRow->addWidget(clear);attachRow->addWidget(attachmentLabel_,1);ml->addLayout(attachRow);
 auto *row=new QHBoxLayout;input_=new QLineEdit;input_->setPlaceholderText("Ask Doctor AI…");performance_=new QComboBox;performance_->addItem("Fast","fast");performance_->addItem("Balanced","balanced");performance_->addItem("Quality","quality");performance_->setCurrentIndex(1);send_=new QPushButton("Send  ➜");send_->setObjectName("primary");row->addWidget(input_,1);row->addWidget(performance_);row->addWidget(send_);ml->addLayout(row);
 outer->addWidget(side);outer->addWidget(main,1);setCentralWidget(root);
 setStyleSheet(R"(
 QMainWindow,QWidget{background:#0b1422;color:#edf4ff;font-family:"Segoe UI";font-size:13px}
 QFrame#sidebar{background:#101d2e;border-right:1px solid #23364c}
 QPushButton{background:#182a3f;color:#dce9fa;border:1px solid #2a4059;border-radius:9px;padding:11px;text-align:left}
 QPushButton:hover{background:#233d58}
 QPushButton#primary{background:#168bd2;border:0;color:white;font-weight:700;text-align:center;padding:12px 22px}
 QPushButton#searchButton{background:#16466a;border:1px solid #2477a9}
 QPushButton#settingsButton{text-align:center}
 QLineEdit,QTextEdit,QComboBox{background:#101e30;border:1px solid #293e57;border-radius:10px;padding:10px;color:#edf4ff}
 QComboBox QAbstractItemView{background:#14243a;color:#edf4ff;selection-background-color:#168bd2}
 )");
 connect(search,&QPushButton::clicked,this,&MainWindow::searchLocalModels);connect(settings,&QPushButton::clicked,this,&MainWindow::openSettings);
 connect(attach,&QPushButton::clicked,this,&MainWindow::addAttachments);connect(clear,&QPushButton::clicked,this,&MainWindow::clearAttachments);
 connect(&api_,&ApiClient::healthChanged,this,[this](bool ready,const QString &m){status_->setText("● "+m);status_->setStyleSheet(ready?"color:#49d6a2":"color:#ffb36b");});
 connect(&api_,&ApiClient::modelsReceived,this,[this](const QStringList &names,const QString &active){models_->clear();models_->addItems(names);if(names.isEmpty())models_->addItem("Gemma 4 not installed");int i=models_->findText(active);if(i>=0)models_->setCurrentIndex(i);modelCount_->setText(names.isEmpty()?"Offline model: not installed":"Offline model ready");});
 connect(&api_,&ApiClient::answerReceived,this,[this](const QString &a){append("Doctor AI",a);setBusy(false);});
 connect(&api_,&ApiClient::failed,this,[this](const QString &e){append("System",e);setBusy(false);});
 connect(send_,&QPushButton::clicked,this,[this]{QString message=input_->text().trimmed();if(message.isEmpty()&&attachments_.isEmpty())return;if(models_->count()==0||models_->currentText()=="Gemma 4 not installed"){QMessageBox::warning(this,"Offline model missing","Run the Doctor AI installer again and select the recommended Gemma 4 model.");return;}append("You",message.isEmpty()?"Please analyze the attached files.":message);input_->clear();setBusy(true);api_.sendMessage(message,models_->currentText(),attachments_,performance_->currentData().toString());});
 connect(input_,&QLineEdit::returnPressed,send_,&QPushButton::click);
 api_.checkHealth();api_.fetchModels();
 auto *healthTimer=new QTimer(this);connect(healthTimer,&QTimer::timeout,this,[this]{api_.checkHealth();});healthTimer->start(5000);
}
void MainWindow::append(const QString &speaker,const QString &text){conversation_->append("<b>"+speaker.toHtmlEscaped()+"</b><br>"+text.toHtmlEscaped().replace("\n","<br>")+"<br>");}
void MainWindow::setBusy(bool busy){send_->setEnabled(!busy);input_->setEnabled(!busy);}
void MainWindow::addAttachments(){QStringList files=QFileDialog::getOpenFileNames(this,"Attach images or text files",QString(),"Supported files (*.png *.jpg *.jpeg *.webp *.bmp *.txt *.md *.csv *.log *.pdf *.docx *.mp4 *.mov *.avi *.mkv *.webm);;All files (*.*)");for(const QString &path:files)if(!attachments_.contains(path))attachments_.append(path);QStringList names;for(const QString &path:attachments_)names<<QFileInfo(path).fileName();attachmentLabel_->setText(names.isEmpty()?"No attachments":names.join(", "));attachmentLabel_->setToolTip(attachments_.join("\n"));}
void MainWindow::clearAttachments(){attachments_.clear();attachmentLabel_->setText("No attachments");attachmentLabel_->setToolTip("");}
void MainWindow::searchLocalModels(){status_->setText("● Checking installed offline model…");api_.checkHealth();api_.fetchModels();}
void MainWindow::openSettings(){
 QDialog d(this);d.setWindowTitle("Offline modules and hardware guide");d.setMinimumWidth(620);auto *l=new QVBoxLayout(&d);
 auto *title=new QLabel("Offline modules");title->setStyleSheet("font-size:20px;font-weight:750");l->addWidget(title);
 auto *info=new QLabel(
 "<b>Available in this build</b><br>• Gemma 4 E2B / E4B — local chat and medical text assistance. E2B is lighter; E4B is stronger and needs more RAM.<br>• Image understanding — optional Gemma 4 projector; download it during setup.<br><br>"
 "<b>Catalog only — not installed in this build</b><br>• Whisper.cpp — offline speech-to-text (8 GB RAM recommended).<br>• Piper — offline text-to-speech (4 GB RAM).<br>• Stable Diffusion / ComfyUI — image generation (16 GB RAM + 8 GB dedicated VRAM recommended).<br>• Wan 2.x / ComfyUI — video generation (32 GB RAM + 16–24 GB VRAM recommended).<br>• Offline audio generation — 32 GB RAM and a capable GPU recommended.<br><br>"
 "<b>Hardware guide</b><br>4 GB RAM: desktop shell only; offline AI not recommended.<br>8–15 GB RAM: Gemma 4 E2B Q4_0.<br>16 GB RAM or more: Gemma 4 E4B Q4_0.<br>GPU acceleration is not included in this CPU-only build.<br><br>"
 "Java and Microsoft C++ runtime are bundled. No Ollama, Python, Docker, or separate Java install is required.");
 info->setWordWrap(true);info->setTextFormat(Qt::RichText);l->addWidget(info);
 auto *buttons=new QDialogButtonBox(QDialogButtonBox::Close);l->addWidget(buttons);connect(buttons,&QDialogButtonBox::rejected,&d,&QDialog::reject);d.exec();
}