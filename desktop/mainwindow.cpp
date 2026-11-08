#include "mainwindow.h"
#include <QHBoxLayout>
#include <QVBoxLayout>
#include <QFrame>
#include <QMessageBox>
#include <QDialog>
#include <QFormLayout>
#include <QDialogButtonBox>
#include <QRegularExpression>

MainWindow::MainWindow(QWidget *parent):QMainWindow(parent){
    setWindowTitle("Doctor AI");
    resize(1240,820);
    setMinimumSize(960,650);

    auto *root=new QWidget;
    auto *outer=new QHBoxLayout(root);
    outer->setContentsMargins(0,0,0,0);
    outer->setSpacing(0);

    auto *side=new QFrame;
    side->setObjectName("sidebar");
    side->setFixedWidth(245);
    auto *sl=new QVBoxLayout(side);
    sl->setContentsMargins(22,26,22,22);

    auto *brand=new QLabel("✚  DOCTOR AI");
    brand->setStyleSheet("font-size:22px;font-weight:800;color:#55c2ff");
    sl->addWidget(brand);
    auto *tag=new QLabel("LOCAL INTELLIGENCE WORKSPACE");
    tag->setStyleSheet("font-size:9px;color:#8296ae;letter-spacing:1px");
    sl->addWidget(tag);
    sl->addSpacing(30);

    auto *section=new QLabel("WORKSPACE");
    section->setStyleSheet("color:#8195ad;font-size:10px;font-weight:700");
    sl->addWidget(section);
    for(const QString &s:QStringList{"General assistant","Medical assistant","Image studio","Video studio","Audio studio"}){
        auto *b=new QPushButton("   "+s);
        b->setCursor(Qt::PointingHandCursor);
        b->setEnabled(s=="General assistant"||s=="Medical assistant");
        sl->addWidget(b);
    }

    sl->addStretch();
    status_=new QLabel("● Connecting to local services…");
    status_->setWordWrap(true);
    sl->addWidget(status_);
    modelCount_=new QLabel("Local models: checking…");
    modelCount_->setStyleSheet("color:#8296ae;font-size:10px");
    sl->addWidget(modelCount_);
    auto *settings=new QPushButton("⚙  Settings");
    settings->setObjectName("settingsButton");
    settings->setCursor(Qt::PointingHandCursor);
    sl->addWidget(settings);

    auto *main=new QWidget;
    auto *ml=new QVBoxLayout(main);
    ml->setContentsMargins(30,25,30,24);
    ml->setSpacing(14);

    auto *top=new QHBoxLayout;
    auto *heading=new QLabel("Your assistant");
    heading->setStyleSheet("font-size:25px;font-weight:750");
    top->addWidget(heading);
    top->addStretch();

    models_=new QComboBox;
    models_->setMinimumWidth(220);
    models_->addItem("Searching local models…");
    top->addWidget(new QLabel("MODEL"));
    top->addWidget(models_);

    auto *search=new QPushButton("⌕  Search local models");
    search->setObjectName("searchButton");
    search->setCursor(Qt::PointingHandCursor);
    top->addWidget(search);
    ml->addLayout(top);

    auto *sub=new QLabel("Private by design · Model inference runs through your local Ollama service");
    sub->setStyleSheet("color:#8fa4bd");
    ml->addWidget(sub);

    conversation_=new QTextEdit;
    conversation_->setReadOnly(true);
    conversation_->setPlaceholderText("Welcome to Doctor AI. Search for locally installed models, choose one, and start a conversation.");
    ml->addWidget(conversation_,1);

    auto *hint=new QLabel("LOCAL MODEL CHAT  ·  Never use AI output as the sole basis for clinical decisions.");
    hint->setStyleSheet("color:#8296ae;font-size:10px");
    ml->addWidget(hint);

    auto *row=new QHBoxLayout;
    input_=new QLineEdit;
    input_->setPlaceholderText("Ask Doctor AI…");
    send_=new QPushButton("Send  ➜");
    send_->setObjectName("primary");
    row->addWidget(input_,1);
    row->addWidget(send_);
    ml->addLayout(row);

    outer->addWidget(side);
    outer->addWidget(main,1);
    setCentralWidget(root);

    setStyleSheet(R"(
      QMainWindow,QWidget{background:#0b1422;color:#edf4ff;font-family:"Segoe UI";font-size:13px}
      QFrame#sidebar{background:#101d2e;border-right:1px solid #23364c}
      QPushButton{background:#182a3f;color:#dce9fa;border:1px solid #2a4059;border-radius:9px;padding:11px;text-align:left}
      QPushButton:hover{background:#233d58}
      QPushButton#primary{background:#168bd2;border:0;color:white;font-weight:700;text-align:center;padding:12px 22px}
      QPushButton#searchButton{background:#16466a;border:1px solid #2477a9}
      QPushButton#settingsButton{text-align:center}
      QLineEdit,QTextEdit,QComboBox{background:#101e30;border:1px solid #293e57;border-radius:10px;padding:12px;color:#edf4ff}
      QComboBox QAbstractItemView{background:#14243a;color:#edf4ff;selection-background-color:#168bd2}
    )");

    connect(search,&QPushButton::clicked,this,&MainWindow::searchLocalModels);
    connect(settings,&QPushButton::clicked,this,&MainWindow::openSettings);
    connect(&api_,&ApiClient::healthChanged,this,[this](bool ok,const QString &m){
        status_->setText(ok?"● Ollama connected":"● "+m);
        status_->setStyleSheet(ok?"color:#49d6a2":"color:#ff8b8b");
    });
    connect(&api_,&ApiClient::modelsReceived,this,[this](const QStringList &names,const QString &active){
        models_->clear();
        models_->addItems(names);
        if(names.isEmpty()) models_->addItem("No local models found");
        int i=models_->findText(active);
        if(i>=0) models_->setCurrentIndex(i);
        modelCount_->setText(QString("Local models found: %1").arg(names.size()));
        if(names.isEmpty()) status_->setText("● No local models detected");
    });
    connect(&api_,&ApiClient::settingsReceived,this,[this](const QString &url){
        status_->setToolTip("Ollama: "+url);
    });
    connect(&api_,&ApiClient::settingsSaved,this,[this](bool ok,const QString &message){
        QMessageBox::information(this,ok?"Settings saved":"Settings error",message);
        if(ok){api_.checkHealth();api_.fetchModels();}
    });
    connect(&api_,&ApiClient::answerReceived,this,[this](const QString &a){append("Doctor AI",a);setBusy(false);});
    connect(&api_,&ApiClient::failed,this,[this](const QString &e){append("System",e);setBusy(false);});

    connect(send_,&QPushButton::clicked,this,[this]{
        const QString message=input_->text().trimmed();
        if(message.isEmpty()) return;
        if(models_->count()==0 || models_->currentText()=="No local models found"){
            QMessageBox::warning(this,"No local model","Install a model with Ollama, then use Search local models.");
            return;
        }
        append("You",message);
        input_->clear();
        setBusy(true);
        api_.sendMessage(message,models_->currentText());
    });
    connect(input_,&QLineEdit::returnPressed,send_,&QPushButton::click);

    api_.fetchSettings();
    api_.checkHealth();
    api_.fetchModels();
}

void MainWindow::append(const QString &speaker,const QString &text){
    conversation_->append("<b>"+speaker.toHtmlEscaped()+"</b><br>"+text.toHtmlEscaped().replace("\n","<br>")+"<br>");
}
void MainWindow::setBusy(bool busy){send_->setEnabled(!busy);input_->setEnabled(!busy);}

void MainWindow::searchLocalModels(){
    QMessageBox prompt(this);
    prompt.setWindowTitle("Search local AI models");
    prompt.setText("Doctor AI will scan your configured local Ollama server for installed models.");
    prompt.setInformativeText("This checks the local Ollama /api/tags endpoint. It does not download models or upload your files. Make sure Ollama is running, then press Search now.");
    prompt.setStandardButtons(QMessageBox::Cancel|QMessageBox::Yes);
    prompt.button(QMessageBox::Yes)->setText("Search now");
    if(prompt.exec()==QMessageBox::Yes){
        status_->setText("● Searching installed local models…");
        modelCount_->setText("Local models: scanning…");
        api_.checkHealth();
        api_.fetchModels();
    }
}

void MainWindow::openSettings(){
    QDialog dialog(this);
    dialog.setWindowTitle("Doctor AI settings");
    dialog.setMinimumWidth(460);
    auto *layout=new QVBoxLayout(&dialog);
    auto *form=new QFormLayout;
    auto *ollamaUrl=new QLineEdit;
    ollamaUrl->setPlaceholderText("http://127.0.0.1:11434");
    ollamaUrl->setText(status_->toolTip().startsWith("Ollama: ")?status_->toolTip().mid(8):"http://127.0.0.1:11434");
    form->addRow("Ollama server URL",ollamaUrl);
    layout->addLayout(form);
    auto *note=new QLabel("Use the address of your local Ollama server. The setting is saved on this computer and used for model discovery and chat.");
    note->setWordWrap(true);
    note->setStyleSheet("color:#91a7c0");
    layout->addWidget(note);
    auto *buttons=new QDialogButtonBox(QDialogButtonBox::Save|QDialogButtonBox::Cancel);
    layout->addWidget(buttons);
    connect(buttons,&QDialogButtonBox::rejected,&dialog,&QDialog::reject);
    connect(buttons,&QDialogButtonBox::accepted,&dialog,&QDialog::accept);
    if(dialog.exec()==QDialog::Accepted){
        const QString value=ollamaUrl->text().trimmed();
        QUrl parsed(value);
        if(!parsed.isValid() || parsed.host().isEmpty() || (parsed.scheme()!="http" && parsed.scheme()!="https")){
            QMessageBox::warning(this,"Invalid URL","Enter a valid address such as http://127.0.0.1:11434");
            return;
        }
        api_.saveOllamaUrl(value);
    }
}
