#include "mainwindow.h"
#include <QApplication>
int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    QApplication::setApplicationName("Doctor AI");
    QApplication::setOrganizationName("DoctorAI");
    QApplication::setStyle("Fusion");
    MainWindow window;
    window.show();
    return app.exec();
}
