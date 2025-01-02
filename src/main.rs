use pcsc::{Context, Error, Protocols, Scope, ShareMode, MAX_BUFFER_SIZE};
use hex_literal::hex;

fn main() {
    let ctx = match Context::establish(Scope::User) {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Failed to establish context: {}", err);
            std::process::exit(1);
        }
    };

    // List available readers.
    let mut readers_buf = [0; 2048];
    let mut readers = match ctx.list_readers(&mut readers_buf) {
        Ok(readers) => readers,
        Err(err) => {
            eprintln!("Failed to list readers: {}", err);
            std::process::exit(1);
        }
    };

    // Use the first reader.
    let reader = match readers.next() {
        Some(reader) => reader,
        None => {
            println!("No readers are connected.");
            return;
        }
    };
    println!("Using reader: {:?}", reader);

    // Connect to the card.
    let card = match ctx.connect(reader, ShareMode::Shared, Protocols::ANY) {
        Ok(card) => card,
        Err(Error::NoSmartcard) => {
            println!("A smartcard is not present in the reader.");
            return;
        }
        Err(err) => {
            eprintln!("Failed to connect to card: {}", err);
            std::process::exit(1);
        }
    };

    // Class byteはこちら
    // https://cardwerk.com/smart-card-standard-iso7816-4-section-5-basic-organizations/?elementor-preview&1514396438071#chap5_4_1
    let apdu_command_of_get_data = hex!("FF CA 00 00 00");
    let mut rapdu_buf = [0; MAX_BUFFER_SIZE];

    // transmitで、 rapdu_buf に値が設定される
    // 構造は、 DataField + SW1 + SW2
    // https://cardwerk.com/smart-card-standard-iso7816-4-section-6-basic-interindustry-commands/
    // DataFieldの長さは、戻り値のlenを取れば出てくる
    // SW1とSW2は、ステータスバイトなので、それぞれ1バイトずつの合計2バイト
    let rapdu = card.transmit(&apdu_command_of_get_data, &mut rapdu_buf).unwrap();

    // rapduとrapdu_bufには同じ値が入ってきてるっぽい感じがある
    println!("{rapdu:02X?}");

    // データ部とステータス部を分割する
    // ステータス部は最低2バイトあるので、その長さ未満の場合は、なにかおかしいと判断する
    let response_length = rapdu.len();
    if response_length < 2 {
        println!("レスポンスが不正です");
        return
    }

    // ステータス部は2バイトで固定のため、それ以外の部分はデータ部として判断する
    let (data, sw) = rapdu.split_at(response_length - 2);

    if sw[0] == 0x90 && sw[1] == 0x00 {
        println!("IDmを取得できました");
        println!("{data:02X?}");
    } else {
        println!("IDmを取得できませんでした");
    }
}
