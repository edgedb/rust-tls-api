use crate::AsyncSocket;
use crate::AsyncSocketBox;
use crate::BoxFuture;
use crate::Cert;
use crate::Pkcs12AndPassword;
use crate::PrivateKey;
use crate::TlsAcceptor;
use crate::TlsAcceptorBuilder;
use crate::TlsStreamBox;
use std::marker;

// Type

/// [`TlsAcceptor`] as dynamic object.
///
/// Created with [`TlsAcceptor::TYPE_DYN`].
pub trait TlsAcceptorType {
    /// Whether this acceptor type is implemented.
    ///
    /// For example, `tls-api-security-framework` is available on Linux,
    /// but all operations result in error, so `IMPLEMENTED = false`
    /// for that implementation.
    fn implemented(&self) -> bool;
    /// Whether this implementation supports ALPN negotiation.
    fn supports_alpn(&self) -> bool;
    /// Whether this implementation supports construction of acceptor using
    /// a pair of a DER certificate and file pair.
    fn supports_der_keys(&self) -> bool;
    /// Whether this implementation supports construction of acceptor using
    /// PKCS #12 file.
    fn supports_pkcs12_keys(&self) -> bool;
    /// Unspecified version information about this implementation.
    fn version(&self) -> &'static str;

    /// New builder from given server key.
    ///
    /// This operation is guaranteed to fail if not [`TlsAcceptorType::supports_der_keys`].
    fn builder_from_der_key(
        &self,
        cert: &Cert,
        key: &PrivateKey,
    ) -> crate::Result<TlsAcceptorBuilderBox>;

    /// New builder from given server key.
    ///
    /// This operation is guaranteed to fail if not [`TlsAcceptorType::supports_pkcs12_keys`].
    fn builder_from_pkcs12(
        &self,
        pkcs12: &Pkcs12AndPassword,
    ) -> crate::Result<TlsAcceptorBuilderBox>;
}

pub(crate) struct TlsAcceptorTypeImpl<A: TlsAcceptor>(pub marker::PhantomData<A>);

impl<A: TlsAcceptor> TlsAcceptorType for TlsAcceptorTypeImpl<A> {
    fn implemented(&self) -> bool {
        A::IMPLEMENTED
    }

    fn supports_alpn(&self) -> bool {
        A::SUPPORTS_ALPN
    }

    fn supports_der_keys(&self) -> bool {
        A::SUPPORTS_DER_KEYS
    }

    fn supports_pkcs12_keys(&self) -> bool {
        A::SUPPORTS_PKCS12_KEYS
    }

    fn version(&self) -> &'static str {
        A::version()
    }

    fn builder_from_der_key(
        &self,
        cert: &Cert,
        key: &PrivateKey,
    ) -> crate::Result<TlsAcceptorBuilderBox> {
        let builder = A::builder_from_der_key(cert, key)?;
        Ok(TlsAcceptorBuilderBox(Box::new(builder)))
    }

    fn builder_from_pkcs12(
        &self,
        pkcs12: &Pkcs12AndPassword,
    ) -> crate::Result<TlsAcceptorBuilderBox> {
        let builder = A::builder_from_pkcs12(pkcs12)?;
        Ok(TlsAcceptorBuilderBox(Box::new(builder)))
    }
}

// Builder

trait TlsAcceptorBuilderDyn {
    fn type_dyn(&self) -> &'static dyn TlsAcceptorType;

    fn set_alpn_protocols(&mut self, protocols: &[&[u8]]) -> crate::Result<()>;

    fn build(self: Box<Self>) -> crate::Result<TlsAcceptorBox>;
}

impl<A: TlsAcceptorBuilder> TlsAcceptorBuilderDyn for A {
    fn type_dyn(&self) -> &'static dyn TlsAcceptorType {
        <A::Acceptor as TlsAcceptor>::TYPE_DYN
    }

    fn set_alpn_protocols(&mut self, protocols: &[&[u8]]) -> crate::Result<()> {
        (*self).set_alpn_protocols(protocols)
    }

    fn build(self: Box<Self>) -> crate::Result<TlsAcceptorBox> {
        Ok(TlsAcceptorBox(Box::new((*self).build()?)))
    }
}

/// Dynamic version of [`TlsAcceptorBuilder`].
pub struct TlsAcceptorBuilderBox(Box<dyn TlsAcceptorBuilderDyn>);

impl TlsAcceptorBuilderBox {
    /// Dynamic (without type parameter) version of the acceptor.
    ///
    /// This function returns a connector type, which can be used to constructor connectors.
    pub fn type_dyn(&self) -> &'static dyn TlsAcceptorType {
        self.0.type_dyn()
    }

    /// Specify ALPN protocols for negotiation.
    ///
    /// This operation returns an error if the implemenation does not support ALPN.
    ///
    /// Whether ALPN is supported, can be queried using [`TlsAcceptor::SUPPORTS_ALPN`].
    pub fn set_alpn_protocols(&mut self, protocols: &[&[u8]]) -> crate::Result<()> {
        self.0.set_alpn_protocols(protocols)
    }

    /// Finish the acceptor construction.
    pub fn build(self) -> crate::Result<TlsAcceptorBox> {
        self.0.build()
    }
}

// Acceptor

trait TlsAcceptorDyn {
    fn type_dyn(&self) -> &'static dyn TlsAcceptorType;

    fn accept<'a>(&'a self, socket: AsyncSocketBox) -> BoxFuture<'a, crate::Result<TlsStreamBox>>;
}

impl<A: TlsAcceptor> TlsAcceptorDyn for A {
    fn type_dyn(&self) -> &'static dyn TlsAcceptorType {
        A::TYPE_DYN
    }

    fn accept<'a>(&'a self, socket: AsyncSocketBox) -> BoxFuture<'a, crate::Result<TlsStreamBox>> {
        self.accept_dyn(socket)
    }
}

/// Dynamic version of [`TlsAcceptor`].
///
/// This can be constructed either with:
/// * [`TlsAcceptor::into_dyn`]
/// * [`TlsAcceptorBuilderBox::build`]
pub struct TlsAcceptorBox(Box<dyn TlsAcceptorDyn>);

impl TlsAcceptorBox {
    pub(crate) fn new<A: TlsAcceptor>(acceptor: A) -> TlsAcceptorBox {
        TlsAcceptorBox(Box::new(acceptor))
    }

    /// Dynamic (without type parameter) version of the acceptor.
    ///
    /// This function returns a connector type, which can be used to constructor connectors.
    pub fn type_dyn(&self) -> &'static dyn TlsAcceptorType {
        self.0.type_dyn()
    }

    /// Accept a connection.
    ///
    /// This operation returns a future which is resolved when the negotiation is complete,
    /// and the stream is ready to send and receive.
    pub fn accept<'a, S: AsyncSocket>(
        &'a self,
        socket: S,
    ) -> BoxFuture<'a, crate::Result<TlsStreamBox>> {
        self.0.accept(AsyncSocketBox::new(socket))
    }
}