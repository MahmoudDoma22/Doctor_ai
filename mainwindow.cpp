#include "mainwindow.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QFileDialog>
#include <QFileInfo>
#include <QMessageBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QTextEdit>
#include <QFormLayout>
#include <QFrame>
#include <QJsonDocument>
#include <QJsonObject>

MainWindow::MainWindow(QWidget *parent) : QMainWindow(parent) {
    setWindowTitle("Doctor AI");
    resize(1180, 760);
    setMinimumSize(900, 620);
    pages_ = new QStackedWidget(this);
    setCentralWidget(pages_);
    pages_->addWidget(makeWelcome());
    pages_->addWidget(makeConnect());
    pages_->addWidget(makeChat());

    connect(&api_, &ApiClient::healthResult, this, [this](bool ok, const QString &msg) {
        status_->setText(ok ? "● Connected to Doctor AI backend" : "● Backend unavailable");
        status_->setStyleSheet(ok ? "color:#43d9a3" : "color:#ff7777");
        if (!ok) QMessageBox::warning(this, "Connection", "Cannot reach the local Java service.\nStart it with: mvn spring-boot:run\n\n" + msg);
    });
    connect(&api_, &ApiClient::chatResult, this, [this](const QString &answer) {
        appendMessage("Doctor AI", answer);
        composer_->setEnabled(true);
    });
    connect(&api_, &ApiClient::requestError, this, [this](const QString &err) {
        appendMessage("System", "Request failed: " + err);
        composer_->setEnabled(true);
    });
    connect(&api_, &ApiClient::skillSaved, this, [this](bool ok, const QString &message) {
        if (ok) QMessageBox::information(this, "Skills", message);
        else QMessageBox::warning(this, "Skills", "Could not save skill: " + message);
    });
    applyTheme();
}

QWidget *MainWindow::makeWelcome() {
    auto *w = new QWidget;
    auto *layout = new QVBoxLayout(w);
    layout->setContentsMargins(90, 70, 90, 70);
    layout->addStretch();
    auto *logo = new QLabel("✚");
    logo->setAlignment(Qt::AlignCenter);
    logo->setStyleSheet("font-size:64px;color:#54b8ff;font-weight:800;");
    layout->addWidget(logo);
    auto *title = new QLabel("Doctor AI");
    title->setAlignment(Qt::AlignCenter);
    title->setStyleSheet("font-size:42px;font-weight:800;color:#f3f7ff;");
    layout->addWidget(title);
    auto *subtitle = new QLabel("Your private, local AI assistant");
    subtitle->setAlignment(Qt::AlignCenter);
    subtitle->setStyleSheet("font-size:16px;color:#9aacc5;");
    layout->addWidget(subtitle);
    layout->addSpacing(28);
    auto *start = new QPushButton("Get started  →");
    start->setObjectName("primary");
    start->setFixedSize(220, 52);
    auto *row = new QHBoxLayout; row->addStretch(); row->addWidget(start); row->addStretch();
    layout->addLayout(row);
    layout->addStretch();
    connect(start, &QPushButton::clicked, this, [this]() { pages_->setCurrentIndex(1); });
    return w;
}

QWidget *MainWindow::makeConnect() {
    auto *w = new QWidget;
    auto *layout = new QVBoxLayout(w);
    layout->setContentsMargins(100, 80, 100, 80);
    layout->addStretch();
    auto *title = new QLabel("Connect your local AI");
    title->setStyleSheet("font-size:30px;font-weight:700;");
    title->setAlignment(Qt::AlignCenter);
    layout->addWidget(title);
    auto *desc = new QLabel("Doctor AI connects to Ollama running on this PC.");
    desc->setAlignment(Qt::AlignCenter);
    desc->setStyleSheet("color:#9aacc5;font-size:14px;");
    layout->addWidget(desc);
    auto *url = new QLineEdit("http://127.0.0.1:8787");
    url->setPlaceholderText("Java backend URL");
    url->setMaximumWidth(440);
    auto *model = new QLabel("Target model:  gemma4:12b");
    model->setAlignment(Qt::AlignCenter);
    model->setStyleSheet("color:#54b8ff;font-weight:700;padding:12px;");
    auto *connectBtn = new QPushButton("Test connection");
    connectBtn->setObjectName("primary");
    connectBtn->setFixedSize(190, 46);
    auto *continueBtn = new QPushButton("Continue to chat");
    continueBtn->setFixedSize(190, 46);
    auto *center = new QHBoxLayout; center->addStretch(); center->addWidget(url); center->addStretch();
    layout->addSpacing(24); layout->addLayout(center); layout->addWidget(model, 0, Qt::AlignCenter);
    auto *buttons = new QHBoxLayout; buttons->addStretch(); buttons->addWidget(connectBtn); buttons->addWidget(continueBtn); buttons->addStretch();
    layout->addLayout(buttons); layout->addStretch();
    connect(connectBtn, &QPushButton::clicked, this, [this, url]() { api_.setBaseUrl(url->text()); api_.checkHealth(); });
    connect(continueBtn, &QPushButton::clicked, this, [this, url]() {
        api_.setBaseUrl(url->text()); pages_->setCurrentIndex(2); api_.checkHealth();
    });
    return w;
}

QWidget *MainWindow::makeChat() {
    auto *w = new QWidget;
    auto *outer = new QHBoxLayout(w); outer->setContentsMargins(0,0,0,0); outer->setSpacing(0);
    auto *side = new QFrame; side->setObjectName("sidebar"); side->setFixedWidth(230);
    auto *sl = new QVBoxLayout(side); sl->setContentsMargins(18,22,18,18);
    auto *brand = new QLabel("✚  Doctor AI"); brand->setStyleSheet("font-size:20px;font-weight:800;color:#54b8ff;");
    sl->addWidget(brand); sl->addSpacing(22);
    auto *newChat = new QPushButton("+  New conversation"); sl->addWidget(newChat);
    auto *skills = new QPushButton("⚙  Skills & settings"); sl->addWidget(skills);
    sl->addStretch();
    status_ = new QLabel("● Checking connection…"); status_->setWordWrap(true); status_->setStyleSheet("color:#9aacc5;font-size:11px;");
    sl->addWidget(status_);
    auto *main = new QWidget; auto *ml = new QVBoxLayout(main); ml->setContentsMargins(28,22,28,22);
    auto *top = new QHBoxLayout; auto *heading = new QLabel("Assistant"); heading->setStyleSheet("font-size:22px;font-weight:700;");
    top->addWidget(heading); top->addStretch();
    auto *model = new QLabel("gemma4:12b  ·  Local"); model->setStyleSheet("color:#54b8ff;padding:8px 12px;background:#172b42;border-radius:8px;");
    top->addWidget(model); ml->addLayout(top);
    chatView_ = new QTextEdit; chatView_->setReadOnly(true); chatView_->setPlaceholderText("Your conversation will appear here…"); ml->addWidget(chatView_,1);
    auto *attachRow = new QHBoxLayout;
    auto *attach = new QPushButton("＋ Attach images");
    auto *attachedLabel = new QLabel("No attachments selected"); attachedLabel->setStyleSheet("color:#91a4bd;font-size:11px;");
    attachRow->addWidget(attach); attachRow->addWidget(attachedLabel); attachRow->addStretch(); ml->addLayout(attachRow);
    auto *sendRow = new QHBoxLayout;
    composer_ = new QLineEdit; composer_->setPlaceholderText("Message Doctor AI…");
    auto *sendBtn = new QPushButton("Send  ➤"); sendBtn->setObjectName("primary");
    sendRow->addWidget(composer_,1); sendRow->addWidget(sendBtn); ml->addLayout(sendRow);
    outer->addWidget(side); outer->addWidget(main,1);
    connect(attach, &QPushButton::clicked, this, [this, attachedLabel]() {
        chooseFiles(); attachedLabel->setText((attachments_.isEmpty() && videos_.isEmpty()) ? "No attachments selected" :
            QString::number(attachments_.size() + videos_.size()) + " file(s) selected");
    });
    connect(sendBtn, &QPushButton::clicked, this, &MainWindow::send);
    connect(composer_, &QLineEdit::returnPressed, this, &MainWindow::send);
    connect(newChat, &QPushButton::clicked, this, [this]() { chatView_->clear(); attachments_.clear(); videos_.clear(); });
    connect(skills, &QPushButton::clicked, this, [this]() {
        QDialog dialog(this);
        dialog.setWindowTitle("Create a Doctor AI skill");
        dialog.setMinimumSize(560, 440);
        auto *form = new QFormLayout(&dialog);
        auto *name = new QLineEdit("clinical-guidance");
        name->setPlaceholderText("e.g. pediatric-emergency");
        auto *content = new QTextEdit;
        content->setPlaceholderText("Write the instructions for this skill in Markdown...");
        form->addRow("Skill name", name);
        form->addRow("Instructions", content);
        auto *buttons = new QDialogButtonBox(QDialogButtonBox::Save | QDialogButtonBox::Cancel);
        form->addRow(buttons);
        connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
        connect(buttons, &QDialogButtonBox::accepted, &dialog, [&]() {
            if (name->text().trimmed().isEmpty() || content->toPlainText().trimmed().isEmpty()) {
                QMessageBox::warning(&dialog, "Missing information", "Enter both a skill name and its instructions.");
                return;
            }
            api_.saveSkill(name->text().trimmed(), content->toPlainText());
            dialog.accept();
        });
        dialog.exec();
    });
    return w;
}

void MainWindow::appendMessage(const QString &who, const QString &text) {
    chatView_->append("<p><b>" + who.toHtmlEscaped() + "</b><br/>" + text.toHtmlEscaped().replace("\n","<br/>") + "</p>");
}
void MainWindow::chooseFiles() {
    const QStringList selected = QFileDialog::getOpenFileNames(this, "Attach media", QString(),
        "Supported media (*.png *.jpg *.jpeg *.webp *.bmp *.mp4 *.mov *.mkv *.avi);;Images (*.png *.jpg *.jpeg *.webp *.bmp);;Videos (*.mp4 *.mov *.mkv *.avi)");
    attachments_.clear(); videos_.clear();
    for (const QString &path : selected) {
        const QString ext = QFileInfo(path).suffix().toLower();
        if (QStringList{"mp4","mov","mkv","avi"}.contains(ext)) videos_.append(path);
        else attachments_.append(path);
    }
}
void MainWindow::send() {
    const QString msg = composer_->text().trimmed();
    if (msg.isEmpty() && attachments_.isEmpty() && videos_.isEmpty()) return;
    appendMessage("You", msg.isEmpty() ? "[Media attachment]" : msg);
    composer_->clear(); composer_->setEnabled(false);
    api_.sendMessage(msg, attachments_, videos_);
    attachments_.clear();
    videos_.clear();
}
void MainWindow::applyTheme() {
    setStyleSheet(R"(
      QMainWindow,QWidget { background:#0b1422;color:#edf4ff;font-family:"Segoe UI";font-size:13px; }
      QFrame#sidebar { background:#101e30;border-right:1px solid #22344b; }
      QPushButton { background:#1a2b40;color:#dce9fa;border:1px solid #2b405a;border-radius:9px;padding:10px 14px; }
      QPushButton:hover { background:#243c57; }
      QPushButton#primary { background:#168bd2;border:0;color:white;font-weight:700; }
      QPushButton#primary:hover { background:#28a4ed; }
      QLineEdit,QTextEdit { background:#101e30;border:1px solid #293e57;border-radius:10px;padding:12px;color:#edf4ff;selection-background-color:#168bd2; }
    )");
}
