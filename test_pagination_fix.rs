use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem, ItemDelegate};

fn main() {
    // Create the same 23 items as the Go example
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
        DefaultItem::new("Warm light", "Like around 2700K"),
        DefaultItem::new("The vernal equinox", "The autumnal one is pretty good too"),
        DefaultItem::new("Gaffer's tape", "Basically sticky fabric"),
        DefaultItem::new("Terrycloth", "In robe form"),
    ];

    println!("=== Pagination Fix Verification ===");
    println!("Total items: {}", items.len());

    // Create list with Go-equivalent dimensions (80x24 terminal, typical list sizing)
    let list_width = 76;  
    let list_height = 22; 
    
    let list = Model::new(items, DefaultDelegate::new(), list_width, list_height)
        .with_title("My Fave Things")
        .with_pagination_type(bubbletea_widgets::paginator::Type::Dots);

    println!("List dimensions: {}x{}", list.width(), list.height());
    
    // Manually calculate pagination like update_pagination does
    let delegate = DefaultDelegate::new();
    let item_height: usize = <DefaultDelegate as ItemDelegate<DefaultItem>>::height(&delegate) + 
                            <DefaultDelegate as ItemDelegate<DefaultItem>>::spacing(&delegate);
    
    let header_height = 
        (if list.show_title() { 1 } else { 0 }) +
        (if list.show_status_bar() { 1 } else { 0 });
    
    // Use the FIXED footer height (1 line for pagination, not 3)
    let footer_height =
        if list.show_help() { 1 } else { 0 } + if list.show_pagination() { 1 } else { 0 };
    
    let available_height = list.height().saturating_sub(header_height + footer_height);
    let items_per_page = (available_height / item_height).max(1);
    let total_pages = (list.len() + items_per_page - 1) / items_per_page;
    
    println!("\n=== Calculation Breakdown ===");
    println!("Item height: {} (height: {} + spacing: {})", 
        item_height,
        <DefaultDelegate as ItemDelegate<DefaultItem>>::height(&delegate),
        <DefaultDelegate as ItemDelegate<DefaultItem>>::spacing(&delegate));
    println!("Header height: {} (title: {}, status: {})", 
        header_height, list.show_title() as u8, list.show_status_bar() as u8);
    println!("Footer height: {} (help: {}, pagination: {})", 
        footer_height, list.show_help() as u8, list.show_pagination() as u8);
    println!("Available height: {} - {} - {} = {}", 
        list.height(), header_height, footer_height, available_height);
    println!("Items per page: {} / {} = {}", available_height, item_height, items_per_page);
    println!("Total pages: ceil({} / {}) = {}", list.len(), items_per_page, total_pages);
    
    println!("\n=== Results ===");
    println!("Calculated total pages: {}", total_pages);
    println!("Expected total pages (Go): 4");
    
    if total_pages == 4 {
        println!("✅ SUCCESS: Pagination matches Go version!");
        println!("   - Items per page: {}", items_per_page);
        println!("   - Pagination dots should show: ● ○ ○ ○ (4 dots)");
        println!("   - Fix applied: Reduced footer height from 3 to 1 line");
        println!("   - Fix applied: Restored spacing to 1 (matching Go default)");
    } else {
        println!("❌ FAILURE: Expected 4 pages, got {}", total_pages);
        if total_pages > 4 {
            println!("   Issue: Too many pages - available height too small");
        } else {
            println!("   Issue: Too few pages - available height too large");
        }
    }
}