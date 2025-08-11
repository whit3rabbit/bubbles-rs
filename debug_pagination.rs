use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem, ItemDelegate};

fn main() {
    // Create items exactly matching the Go example
    let items = vec![
        DefaultItem::new("Raspberry Pi's", "I have 'em all over my house"),
        DefaultItem::new("Nutella", "It's good on toast"),
        DefaultItem::new("Bitter melon", "It cools you down"),
        DefaultItem::new("Nice socks", "And by that I mean socks without holes"),
        DefaultItem::new("Eight hours of sleep", "I had this once"),
        DefaultItem::new("Cats", "Usually"),
        DefaultItem::new("Plantasia, the album", "My plants love it too"),
        DefaultItem::new("Pour over coffee", "It takes forever to make though"),
        DefaultItem::new("VR", "Virtual reality...what is there to say?"),
        DefaultItem::new("Noguchi Lamps", "Such pleasing organic forms"),
        DefaultItem::new("Linux", "Pretty much the best OS"),
        DefaultItem::new("Business school", "Just kidding"),
        DefaultItem::new("Pottery", "Wet clay is a great feeling"),
        DefaultItem::new("Shampoo", "Nothing like clean hair"),
        DefaultItem::new("Table tennis", "It's surprisingly exhausting"),
        DefaultItem::new("Milk crates", "Great for packing in your extra stuff"),
        DefaultItem::new("Afternoon tea", "Especially the tea sandwich part"),
        DefaultItem::new("Stickers", "The thicker the vinyl the better"),
        DefaultItem::new("20° Weather", "Celsius, not Fahrenheit"),
        DefaultItem::new("Warm light", "Like around 2700 Kelvin"),
        DefaultItem::new("The vernal equinox", "The autumnal equinox is pretty good too"),
        DefaultItem::new("Gaffer's tape", "Basically sticky fabric"),
        DefaultItem::new("Terrycloth", "In other words, towel fabric"),
    ];

    let delegate = DefaultDelegate::new();
    
    // Test different terminal sizes
    println!("=== Pagination Debug ===");
    println!("Total items: {}", items.len());
    
    // Try the same dimensions as the example
    let terminal_width = 80;
    let terminal_height = 24;
    let frame_width = 4; // 2 left + 2 right margin from doc_style
    let frame_height = 2; // 1 top + 1 bottom margin from doc_style
    
    let list_width = terminal_width - frame_width;
    let list_height = terminal_height - frame_height;
    
    println!("Terminal size: {}x{}", terminal_width, terminal_height);
    println!("List size: {}x{}", list_width, list_height);
    
    let list = Model::new(items, delegate, list_width, list_height)
        .with_title("My Fave Things")
        .with_pagination_type(bubbletea_widgets::paginator::Type::Dots);
    
    // Use public API methods for debugging
    println!("List height: {}", list.height());
    
    // We can't access per_page directly, but we can infer it from behavior
    let delegate = DefaultDelegate::new();
    
    // Calculate manually like update_pagination does
    let item_height: usize = <DefaultDelegate as ItemDelegate<DefaultItem>>::height(&delegate) + <DefaultDelegate as ItemDelegate<DefaultItem>>::spacing(&delegate);
    let header_height = 
        (if list.show_title() { 1 } else { 0 }) +
        (if list.show_status_bar() { 1 } else { 0 });
    let footer_height =
        if list.show_help() { 1 } else { 0 } + if list.show_pagination() { 1 } else { 0 };
    let available_height = list.height().saturating_sub(header_height + footer_height);
    
    println!("\n=== Manual Calculation (like update_pagination) ===");
    println!("Item height: {} (height: {} + spacing: {})", item_height, <DefaultDelegate as ItemDelegate<DefaultItem>>::height(&delegate), <DefaultDelegate as ItemDelegate<DefaultItem>>::spacing(&delegate));
    println!("Header height: {} (title: {}, status: {})", header_height, list.show_title() as u8, list.show_status_bar() as u8);
    println!("Footer height: {} (help: {}, pagination: {} -> {})", footer_height, list.show_help() as u8, list.show_pagination() as u8, if list.show_pagination() { 1 } else { 0 });
    println!("Available height: {} - {} - {} = {}", list.height(), header_height, footer_height, available_height);
    
    let calculated_per_page = (available_height / item_height).max(1);
    println!("Calculated items per page: {} / {} = {}", available_height, item_height, calculated_per_page);
    
    let items_len = 23; // We know this from the Go example
    let calculated_total_pages = if calculated_per_page > 0 {
        (items_len + calculated_per_page - 1) / calculated_per_page // Ceiling division
    } else {
        1
    };
    println!("Calculated total pages: ceil({} / {}) = {}", items_len, calculated_per_page, calculated_total_pages);
    
    println!("\n=== Go Version Analysis ===");
    println!("If Go shows 4 dots, that means 4 pages total");
    println!("With 23 items, that suggests ~6 items per page: ceil(23/6) = 4");
    println!("So Go might be calculating available height differently");
    
    println!("\n=== Different Height Scenarios ===");
    // Test what height would give us 4 pages
    for test_per_page in 5..=8 {
        let test_pages = (items_len + test_per_page - 1) / test_per_page;
        println!("With {} items per page: {} total pages", test_per_page, test_pages);
    }
    
    println!("\n=== Expected vs Actual ===");
    println!("Expected pages (Go): 4");
    println!("Our calculated pages: {}", calculated_total_pages);
}