// Lấy các công cụ chia bố cục, chỉnh màu và vẽ khối của thư viện ratatui
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
// Lấy cấu trúc App từ file app.rs để đọc dữ liệu
use crate::app::App;

// Hàm chính để vẽ giao diện Menu, nhận vào Khung hình (Frame) và Dữ liệu (App)
pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area(); // Lấy tổng diện tích của màn hình Terminal hiện tại

    // Cắt màn hình ra làm 2 khúc (theo chiều dọc)
    let chunks = Layout::default()
        .direction(Direction::Vertical) // Cắt theo chiều dọc (từ trên xuống)
        .margin(5)                      // Thêm lề xung quanh 5 ô để chữ không sát viền
        .constraints([
            Constraint::Length(3),      // Khúc 1 (cho Tiêu đề): Chiều cao cố định đúng 3 dòng
            Constraint::Min(0)          // Khúc 2 (cho Menu): Chiếm toàn bộ phần không gian còn lại
        ])
        .split(size);                   // Thực hiện cắt trên tổng diện tích `size`

    // --- Vẽ Khúc 1: Tiêu đề ---
    // Tạo một đoạn văn bản làm tiêu đề
    let title = Paragraph::new("🛠️  MY DAILY TOOLS 🛠️")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)) // Đổi màu xanh Cyan, in đậm
        .block(Block::default().borders(Borders::ALL)); // Đóng khung vuông bao quanh
    
    // In tiêu đề đó lên vị trí khúc 1 (chunks[0])
    f.render_widget(title, chunks[0]);

    // --- Vẽ Khúc 2: Danh sách Menu ---
    // Biến mảng chuỗi chữ thành mảng các "ListItem" (định dạng danh sách của Ratatui)
    let items: Vec<ListItem> = app.menu_items.iter()
        .map(|i| ListItem::new(*i).style(Style::default().fg(Color::White))) // Set màu chữ trắng
        .collect();

    // Tạo widget List chứa các ListItem ở trên
    let menu = List::new(items)
        .block(Block::default().title(" Chọn công cụ (Lên/Xuống và Enter) ").borders(Borders::ALL)) // Đóng khung
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)) // Nếu bôi đen: nền xám, in đậm
        .highlight_symbol(">> "); // Dấu mũi tên chỉ vào dòng đang được bôi đen

    // In danh sách ra khúc 2 (chunks[1]), cần truyền app.menu_state để nó biết đang bôi đen dòng nào
    f.render_stateful_widget(menu, chunks[1], &mut app.menu_state);
}