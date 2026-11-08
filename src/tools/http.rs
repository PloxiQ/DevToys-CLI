use crate::error::{Result, ToolError};

#[derive(Debug, Clone, Copy)]
pub struct Status {
    pub code: u16,
    pub reason: &'static str,
    pub description: &'static str,
}

const STATUSES: &[Status] = &[
    Status {
        code: 100,
        reason: "Continue",
        description: "The server received the request headers and the client should send the body.",
    },
    Status {
        code: 101,
        reason: "Switching Protocols",
        description: "The server is switching protocols as requested by the client.",
    },
    Status {
        code: 102,
        reason: "Processing",
        description: "The server has accepted the request but has not finished processing it.",
    },
    Status {
        code: 103,
        reason: "Early Hints",
        description: "The server is sending preliminary headers before the final response.",
    },
    Status {
        code: 200,
        reason: "OK",
        description: "The request succeeded and the response contains the result.",
    },
    Status {
        code: 201,
        reason: "Created",
        description: "The request succeeded and a new resource was created.",
    },
    Status {
        code: 202,
        reason: "Accepted",
        description: "The request was accepted for processing but is not complete.",
    },
    Status {
        code: 203,
        reason: "Non-Authoritative Information",
        description: "The response came from a transformed copy of the origin response.",
    },
    Status {
        code: 204,
        reason: "No Content",
        description: "The request succeeded and there is no response body.",
    },
    Status {
        code: 205,
        reason: "Reset Content",
        description: "The client should reset the document view that sent the request.",
    },
    Status {
        code: 206,
        reason: "Partial Content",
        description: "The server is returning part of the resource for a range request.",
    },
    Status {
        code: 207,
        reason: "Multi-Status",
        description:
            "The response contains status information for multiple independent operations.",
    },
    Status {
        code: 208,
        reason: "Already Reported",
        description:
            "A binding was already reported in an earlier part of a multi-status response.",
    },
    Status {
        code: 226,
        reason: "IM Used",
        description: "The server fulfilled a request using instance manipulations.",
    },
    Status {
        code: 300,
        reason: "Multiple Choices",
        description: "The request has more than one possible response.",
    },
    Status {
        code: 301,
        reason: "Moved Permanently",
        description: "The resource has moved permanently; update links and bookmarks.",
    },
    Status {
        code: 302,
        reason: "Found",
        description: "The resource is temporarily at another URI.",
    },
    Status {
        code: 303,
        reason: "See Other",
        description: "The client should fetch another URI with a GET request.",
    },
    Status {
        code: 304,
        reason: "Not Modified",
        description: "The cached representation is still valid.",
    },
    Status {
        code: 305,
        reason: "Use Proxy",
        description: "The resource must be accessed through the specified proxy (deprecated).",
    },
    Status {
        code: 307,
        reason: "Temporary Redirect",
        description: "The resource is temporarily at another URI; preserve the method and body.",
    },
    Status {
        code: 308,
        reason: "Permanent Redirect",
        description: "The resource is permanently at another URI; preserve the method and body.",
    },
    Status {
        code: 400,
        reason: "Bad Request",
        description: "The server cannot process the request because of client input.",
    },
    Status {
        code: 401,
        reason: "Unauthorized",
        description: "Authentication is required or the supplied credentials are invalid.",
    },
    Status {
        code: 402,
        reason: "Payment Required",
        description: "Payment or another commercial condition is required.",
    },
    Status {
        code: 403,
        reason: "Forbidden",
        description: "The server understands the request but refuses to authorize it.",
    },
    Status {
        code: 404,
        reason: "Not Found",
        description: "The server cannot find the requested resource.",
    },
    Status {
        code: 405,
        reason: "Method Not Allowed",
        description: "The request method is not supported for this resource.",
    },
    Status {
        code: 406,
        reason: "Not Acceptable",
        description: "No representation acceptable to the client is available.",
    },
    Status {
        code: 407,
        reason: "Proxy Authentication Required",
        description: "The client must authenticate with the proxy first.",
    },
    Status {
        code: 408,
        reason: "Request Timeout",
        description: "The server timed out waiting for the request.",
    },
    Status {
        code: 409,
        reason: "Conflict",
        description: "The request conflicts with the current state of the resource.",
    },
    Status {
        code: 410,
        reason: "Gone",
        description: "The resource has been permanently removed and will not return.",
    },
    Status {
        code: 411,
        reason: "Length Required",
        description: "The request must include a Content-Length header.",
    },
    Status {
        code: 412,
        reason: "Precondition Failed",
        description: "A conditional request header evaluated to false.",
    },
    Status {
        code: 413,
        reason: "Content Too Large",
        description: "The request body is larger than the server is willing to accept.",
    },
    Status {
        code: 414,
        reason: "URI Too Long",
        description: "The request target is longer than the server can handle.",
    },
    Status {
        code: 415,
        reason: "Unsupported Media Type",
        description: "The request body format is not supported by the resource.",
    },
    Status {
        code: 416,
        reason: "Range Not Satisfiable",
        description: "The requested byte range cannot be served.",
    },
    Status {
        code: 417,
        reason: "Expectation Failed",
        description: "The server cannot meet the Expect request header.",
    },
    Status {
        code: 418,
        reason: "I'm a teapot",
        description: "The server refuses to brew coffee because it is a teapot.",
    },
    Status {
        code: 421,
        reason: "Misdirected Request",
        description: "The request was sent to a server that cannot produce a response.",
    },
    Status {
        code: 422,
        reason: "Unprocessable Content",
        description: "The request is well-formed but contains semantic errors.",
    },
    Status {
        code: 423,
        reason: "Locked",
        description: "The target resource is locked.",
    },
    Status {
        code: 424,
        reason: "Failed Dependency",
        description: "The request failed because a previous request failed.",
    },
    Status {
        code: 425,
        reason: "Too Early",
        description: "The server is unwilling to risk processing a replayable request.",
    },
    Status {
        code: 426,
        reason: "Upgrade Required",
        description: "The client must switch to a different protocol.",
    },
    Status {
        code: 428,
        reason: "Precondition Required",
        description: "The server requires the request to be conditional.",
    },
    Status {
        code: 429,
        reason: "Too Many Requests",
        description: "The client has sent too many requests in a given time.",
    },
    Status {
        code: 431,
        reason: "Request Header Fields Too Large",
        description: "The request headers are too large for the server.",
    },
    Status {
        code: 451,
        reason: "Unavailable For Legal Reasons",
        description: "The resource is unavailable for legal reasons.",
    },
    Status {
        code: 500,
        reason: "Internal Server Error",
        description: "The server encountered an unexpected condition.",
    },
    Status {
        code: 501,
        reason: "Not Implemented",
        description: "The server does not support the requested functionality.",
    },
    Status {
        code: 502,
        reason: "Bad Gateway",
        description: "An upstream server returned an invalid response.",
    },
    Status {
        code: 503,
        reason: "Service Unavailable",
        description: "The server is temporarily unable to handle the request.",
    },
    Status {
        code: 504,
        reason: "Gateway Timeout",
        description: "An upstream server did not respond in time.",
    },
    Status {
        code: 505,
        reason: "HTTP Version Not Supported",
        description: "The server does not support the HTTP version used.",
    },
    Status {
        code: 506,
        reason: "Variant Also Negotiates",
        description: "Content negotiation resulted in a circular reference.",
    },
    Status {
        code: 507,
        reason: "Insufficient Storage",
        description: "The server cannot store the representation needed to complete the request.",
    },
    Status {
        code: 508,
        reason: "Loop Detected",
        description: "The server detected an infinite loop while processing the request.",
    },
    Status {
        code: 510,
        reason: "Not Extended",
        description: "Further extensions to the request are required.",
    },
    Status {
        code: 511,
        reason: "Network Authentication Required",
        description: "The client must authenticate to gain network access.",
    },
];

pub fn lookup(code: u16) -> Result<Status> {
    STATUSES
        .iter()
        .copied()
        .find(|status| status.code == code)
        .ok_or_else(|| {
            ToolError::new(format!("HTTP status {code} is not in the offline table.")).with_hint(
                "Try a standard code such as 200, 301, 404 or 500, or run `devtoys http --list`.",
            )
        })
}

pub fn list(class: Option<u16>) -> Result<Vec<Status>> {
    if let Some(class) = class {
        if !(1..=5).contains(&class) {
            return Err(ToolError::new("HTTP status class must be between 1 and 5."));
        }
    }
    Ok(STATUSES
        .iter()
        .copied()
        .filter(|status| class.is_none_or(|class| status.code / 100 == class))
        .collect())
}

#[must_use]
pub fn category(code: u16) -> &'static str {
    match code / 100 {
        1 => "1xx Informational",
        2 => "2xx Success",
        3 => "3xx Redirection",
        4 => "4xx Client Error",
        5 => "5xx Server Error",
        _ => "Unknown",
    }
}

#[must_use]
pub fn format(status: Status) -> String {
    format!(
        "Status     : {} {}\nClass      : {}\nMeaning    : {}",
        status.code,
        status.reason,
        category(status.code),
        status.description
    )
}

#[cfg(test)]
mod tests {
    use super::{format, list, lookup};

    #[test]
    fn finds_common_statuses() -> Result<(), Box<dyn std::error::Error>> {
        let status = lookup(404)?;
        assert_eq!(status.reason, "Not Found");
        assert!(format(status).contains("4xx Client Error"));
        assert!(lookup(999).is_err());
        Ok(())
    }

    #[test]
    fn lists_classes() -> Result<(), Box<dyn std::error::Error>> {
        let client_errors = list(Some(4))?;
        assert!(client_errors.iter().all(|status| status.code / 100 == 4));
        assert!(client_errors.iter().any(|status| status.code == 404));
        assert!(list(Some(9)).is_err());
        Ok(())
    }
}
