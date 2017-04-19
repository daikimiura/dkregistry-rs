//! Media-types for API objects.

use futures;
use hyper::{header, mime};
use errors::*;
use strum::EnumProperty;

pub type FutureMediaType = Box<futures::Future<Item = Option<MediaTypes>, Error = Error>>;

// For schema1 types, see https://docs.docker.com/registry/spec/manifest-v2-1/
// For schema2 types, see https://docs.docker.com/registry/spec/manifest-v2-2/

#[derive(EnumProperty,EnumString,ToString,Debug,Hash,PartialEq)]
pub enum MediaTypes {
    /// Manifest, version 2 schema 1.
    #[strum(serialize="application/vnd.docker.distribution.manifest.v1+json")]
    #[strum(props(Sub="vnd.docker.distribution.manifest.v1+json"))]
    ManifestV2S1,
    /// Signed manifest, version 2 schema 1.
    #[strum(serialize="application/vnd.docker.distribution.manifest.v1+prettyjws")]
    #[strum(props(Sub="vnd.docker.distribution.manifest.v1+prettyjws"))]
    ManifestV2S1Signed,
    /// Manifest, version 2 schema 1.
    #[strum(serialize="application/vnd.docker.distribution.manifest.v2+json")]
    #[strum(props(Sub="vnd.docker.distribution.manifest.v2+json"))]
    ManifestV2S2,
    /// Manifest List (aka "fat manifest").
    #[strum(serialize="application/vnd.docker.distribution.manifest.list.v2+json")]
    #[strum(props(Sub="vnd.docker.distribution.manifest.list.v2+json"))]
    ManifestList,
    /// Image layer, as a gzip-compressed tar.
    #[strum(serialize="application/vnd.docker.image.rootfs.diff.tar.gzip")]
    #[strum(props(Sub="vnd.docker.image.rootfs.diff.tar.gzip"))]
    ImageLayerTgz,
    /// Configuration object for a container.
    #[strum(serialize="application/vnd.docker.container.image.v1+json")]
    #[strum(props(Sub="vnd.docker.container.image.v1+json"))]
    ContainerConfigV1,
    /// Generic JSON
    #[strum(serialize="application/json")]
    #[strum(props(Sub="json"))]
    ApplicationJson,
}

impl MediaTypes {
    // TODO(lucab): proper error types
    pub fn from_mime(mtype: &mime::Mime) -> Result<Self> {
        match *mtype {
            mime::Mime(mime::TopLevel::Application, mime::SubLevel::Json, _) => {
                Ok(MediaTypes::ApplicationJson)
            }
            mime::Mime(mime::TopLevel::Application, mime::SubLevel::Ext(ref s), _) => {
                match s.as_str() {
                    "vnd.docker.distribution.manifest.v1+json" => Ok(MediaTypes::ManifestV2S1),
                    "vnd.docker.distribution.manifest.v1+prettyjws" => {
                        Ok(MediaTypes::ManifestV2S1Signed)
                    }
                    "vnd.docker.distribution.manifest.v2+json" => {
                        Ok(MediaTypes::ManifestV2S2)
                    }
                    "vnd.docker.distribution.manifest.list.v2+json" => Ok(MediaTypes::ManifestList),
                    "vnd.docker.image.rootfs.diff.tar.gzip" => Ok(MediaTypes::ImageLayerTgz),
                    "vnd.docker.container.image.v1+json" => Ok(MediaTypes::ContainerConfigV1),
                    _ => bail!("unknown sublevel in mediatype {:?}", mtype),
                }
            }
            _ => bail!("unknown mediatype {:?}", mtype),
        }
    }
    pub fn to_mime(&self) -> mime::Mime {
        match self {
            &MediaTypes::ApplicationJson => mime!(Application / Json),
            ref m => {
                if let Some(s) = m.get_str("Sub") {
                    mime::Mime(mime::TopLevel::Application,
                               mime::SubLevel::Ext(s.to_string()),
                               vec![])
                } else {
                    mime!(Application / Star)
                }
            }
        }
    }
    pub fn to_qitem(&self) -> header::QualityItem<mime::Mime> {
        header::qitem(self.to_mime())
    }
}
