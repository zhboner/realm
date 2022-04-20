use std::io::Result;
use futures::try_join;
use tokio::io::copy_bidirectional;
use kaminari::{AsyncAccept, AsyncConnect, IOStream};
use kaminari::mix::{MixAccept, MixConnect};

pub async fn relay_transport<S: IOStream>(
    src: S,
    dst: S,
    ac: &MixAccept,
    cc: &MixConnect,
) -> Result<()> {
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

pub async fn handshake_and_relay<S, AC, CC>(
    src: S,
    dst: S,
    ac: &AC,
    cc: &CC,
) -> Result<()>
where
    S: IOStream,
    AC: AsyncAccept<S>,
    CC: AsyncConnect<S>,
{
    let (mut src, mut dst) = try_join!(ac.accept(src), cc.connect(dst))?;

    copy_bidirectional(&mut src, &mut dst).await.map(|_| ())
}
