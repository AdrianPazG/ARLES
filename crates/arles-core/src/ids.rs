//! Identificadores tipados.
//!
//! Ver `documentacion/03-arquitectura/MODELO_DE_DATOS.md` §1.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CoreError;

/// Define un identificador con su propio tipo.
///
/// Que `ContactId` y `CampaignId` sean tipos distintos hace que el compilador
/// rechace pasar uno donde va el otro. En una tabla como `message_attempt`, que
/// referencia campaña, contacto y cuenta a la vez, eso no es ceremonia: es la
/// diferencia entre un error de compilación y un correo al destinatario
/// equivocado.
macro_rules! id_tipado {
    ($(#[$meta:meta])* $nombre:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $nombre(Uuid);

        impl $nombre {
            /// Genera uno nuevo.
            ///
            /// UUID **v7**: lleva marca de tiempo en los bits altos, así que
            /// ordena por creación y mantiene los índices compactos. Un v4
            /// aleatorio dispersa las escrituras por todo el índice.
            #[must_use]
            pub fn nuevo() -> Self {
                Self(Uuid::now_v7())
            }

            #[must_use]
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            #[must_use]
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }
        }

        impl fmt::Display for $nombre {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl fmt::Debug for $nombre {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($nombre), "({})"), self.0)
            }
        }

        impl FromStr for $nombre {
            type Err = CoreError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s)
                    .map(Self)
                    .map_err(|_| CoreError::IdInvalido {
                        tipo: stringify!($nombre),
                    })
            }
        }
    };
}

id_tipado!(
    /// Empresa operadora.
    CompanyId
);
id_tipado!(
    /// Cuenta remitente.
    EmailAccountId
);
id_tipado!(
    /// Contacto.
    ContactId
);
id_tipado!(
    /// Canal de un contacto: una dirección de correo o un número de WhatsApp.
    ///
    /// Tiene identidad propia porque un canal se **retira** sin borrarse (queda
    /// con `deleted_at`), y lo retirado se sigue nombrando en el registro de
    /// envíos. Sin id propio no habría forma de distinguir dos direcciones
    /// retiradas del mismo contacto.
    ContactChannelId
);
id_tipado!(
    /// Lote de importación.
    ImportBatchId
);
id_tipado!(
    /// Lista de contactos.
    ContactListId
);
id_tipado!(
    /// Etiqueta.
    TagId
);
id_tipado!(
    /// Plantilla.
    TemplateId
);
id_tipado!(
    /// Versión inmutable de una plantilla.
    TemplateVersionId
);
id_tipado!(
    /// Campaña.
    CampaignId
);
id_tipado!(
    /// Intento de envío.
    MessageAttemptId
);
id_tipado!(
    /// Entrada de supresión.
    SuppressionEntryId
);

/// Clave de idempotencia de un intento de envío.
///
/// Se genera y se persiste **antes** de llamar al proveedor, y viaja como
/// `Message-Id` del correo. Ver `MOTOR_DE_EJECUCION.md` §2 y ADR-0004.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdempotencyKey(Uuid);

impl IdempotencyKey {
    #[must_use]
    pub fn nueva() -> Self {
        Self(Uuid::now_v7())
    }

    /// Forma del `Message-Id` según RFC 5322: `<identificador@dominio>`.
    ///
    /// El dominio lo aporta el remitente, así que el mensaje es rastreable
    /// hasta la cuenta que lo envió.
    ///
    /// Toma una [`EmailAddress`] ya validada y no una cadena suelta: el dominio
    /// acaba en una cabecera SMTP, y con un `&str` cualquiera un CR o LF ahí
    /// permitiría inyectar cabeceras arbitrarias (THREAT_MODEL.md §4.2). Con la
    /// dirección del remitente como entrada, el tipo garantiza la validación.
    #[must_use]
    pub fn como_message_id(&self, remitente: &crate::EmailAddress) -> String {
        format!("<{}@{}>", self.0, remitente.dominio())
    }
}

impl fmt::Display for IdempotencyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for IdempotencyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IdempotencyKey({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_identificadores_son_unicos() {
        let a = ContactId::nuevo();
        let b = ContactId::nuevo();
        assert_ne!(a, b);
    }

    /// Los UUID v7 ordenan por creación: es lo que mantiene los índices
    /// compactos con 500 000 contactos (PRESUPUESTO_RENDIMIENTO.md §3).
    #[test]
    fn los_identificadores_ordenan_por_creacion() {
        let mut anterior = ContactId::nuevo();
        for _ in 0..50 {
            let siguiente = ContactId::nuevo();
            assert!(
                siguiente >= anterior,
                "los UUID v7 deberían ser monótonos: {anterior:?} → {siguiente:?}"
            );
            anterior = siguiente;
        }
    }

    #[test]
    fn ida_y_vuelta_por_texto() {
        let id = CampaignId::nuevo();
        let recuperado: CampaignId = id.to_string().parse().expect("debería parsear");
        assert_eq!(id, recuperado);
    }

    #[test]
    fn rechaza_texto_que_no_es_uuid() {
        assert!("no-soy-un-uuid".parse::<CampaignId>().is_err());
        assert!("".parse::<ContactId>().is_err());
    }

    #[test]
    fn el_message_id_sigue_el_rfc_5322() {
        let remitente = crate::EmailAddress::parse("ventas@empresa.com").expect("válida");
        let clave = IdempotencyKey::nueva();
        let mid = clave.como_message_id(&remitente);
        assert!(mid.starts_with('<'));
        assert!(mid.ends_with("@empresa.com>"));
        assert!(mid.contains(&clave.to_string()));
    }

    /// Hallazgo F5. El `Message-Id` acaba en una cabecera SMTP: si el dominio
    /// pudiera llevar CR o LF, se podrían inyectar cabeceras arbitrarias.
    ///
    /// Exigir una `EmailAddress` ya validada hace que el caso no sea
    /// representable — no hay forma de construir una con saltos de línea.
    #[test]
    fn el_message_id_no_puede_llevar_saltos_de_linea() {
        assert!(
            crate::EmailAddress::parse("ventas@empresa.com\r\nBcc: victima@otra.com").is_err(),
            "si esto se aceptara, el dominio inyectaría cabeceras en el Message-Id"
        );

        let remitente = crate::EmailAddress::parse("ventas@empresa.com").expect("válida");
        let mid = IdempotencyKey::nueva().como_message_id(&remitente);
        assert!(!mid.contains(['\r', '\n']));
    }

    #[test]
    fn serde_serializa_el_uuid_directamente() {
        let id = ContactId::nuevo();
        let json = serde_json::to_string(&id).expect("serializa");
        // `transparent`: sin envoltorio, para que la frontera IPC sea limpia.
        assert_eq!(json, format!("\"{id}\""));
    }
}
