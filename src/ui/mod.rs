// Khai báo 2 file con (menu.rs và dashboard.rs) nằm cùng thư mục để main.rs có thể nhìn thấy
pub mod menu;
pub mod dashboard;

use ratatui::{widgets::{Block, Borders}, Frame};
use crate::app::{App, AppScreen};

// Hàm render tổng được file main.rs gọi. Nó nhận Frame và App.
pub fn render(f: &mut Frame, app: &mut App) {
    // Xét xem biến app.screen hiện tại đang là gì?
    match app.screen {
        // Nếu là Menu, vứt việc vẽ cho file menu.rs lo
        AppScreen::Menu => menu::draw(f, app),
        
        // Nếu là Dashboard, vứt việc vẽ cho file dashboard.rs lo
        AppScreen::Dashboard => dashboard::draw(f, app),
        
        // Nếu là màn hình chưa làm, vẽ tạm một cái khung trống lên toàn màn hình
        AppScreen::PomodoroTool => {
            let block = Block::default().title(" Pomodoro (Chưa làm) - Bấm ESC ").borders(Borders::ALL);
            f.render_widget(block, f.area());
        }
        AppScreen::NotesTool => {
            let block = Block::default().title(" Ghi chú (Chưa làm) - Bấm ESC ").borders(Borders::ALL);
            f.render_widget(block, f.area());
        }
    }
}