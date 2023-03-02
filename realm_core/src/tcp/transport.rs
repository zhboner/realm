use std::io::Result;
use futures::try_join;

use kaminari::{AsyncAccept, AsyncConnect, IOStream};
use kaminari::mix::{MixAccept, MixConnect};

use realm_io::{CopyBuffer, bidi_copy_buf, buf_size};

pub async fn run_relay<S: IOStream>(src: S, dst: S, ac: &MixAccept, cc: &MixConnect) -> Result<()> {
    macro_rules! hs_relay {
        ($ac: expr, $cc: expr) => {
            handshake_and_relay(src, dst, $ac, $cc).await
        };
    }

    #[cfg(feature = "transport-boost")]
    {
        use MixConnect::*;
        if let Some(ac) = ac.as_plain() {
            return match cc {
                Plain(cc) => hs_relay!(ac, cc),
                Ws(cc) => hs_relay!(ac, cc),
                Tls(cc) => hs_relay!(ac, cc),
                Wss(cc) => hs_relay!(ac, cc),
            };
        }
    }

    #[cfg(feature = "transport-boost")]
    {
        use MixAccept::*;
        if let Some(cc) = cc.as_plain() {
            return match ac {
                Plain(ac) => hs_relay!(ac, cc),
                Ws(ac) => hs_relay!(ac, cc),
                Tls(ac) => hs_relay!(ac, cc),
                Wss(ac) => hs_relay!(ac, cc),
            };
        }
    }

    hs_relay!(ac, cc)
}

async fn handshake_and_relay<S, AC, CC>(src: S, dst: S, ac: &AC, cc: &CC) -> Result<()>
where
    S: IOStream,
    AC: AsyncAccept<S>,
    CC: AsyncConnect<S>,
{
    let mut buf1 = vec![0; buf_size()];
    let mut buf2 = vec![0; buf_size()];

    let (mut src, mut dst) = try_join!(ac.accept(src, &mut buf1), cc.connect(dst, &mut buf2))?;

    let buf1 = CopyBuffer::new(buf1);
    let buf2 = CopyBuffer::new(buf2);

    bidi_copy_buf(&mut src, &mut dst, buf1, buf2).await.map(|_| ())
}
