use std::{io, slice, str, fmt};

use bytes::BytesMut;

use httparse;

pub struct Request {
    method: Slice,
    path: Slice,
    // TODO: use a small vec to avoid this unconditional allocation
    headers: Vec<(Slice, Slice)>,
    pub data: BytesMut,
}

type Slice = (usize, usize);

#[derive(Debug)]
pub struct RequestHeaders<'req> {
    headers: slice::Iter<'req, (Slice, Slice)>,
    req: &'req Request,
}

impl Request {
    pub fn method(&self) -> &str {
        str::from_utf8(self.slice(&self.method)).unwrap()
    }

    pub fn path(&self) -> &str {
        str::from_utf8(self.slice(&self.path)).unwrap()
    }

    pub fn headers(&self) -> RequestHeaders {
        RequestHeaders {
            headers: self.headers.iter(),
            req: self,
        }
    }

    fn slice(&self, slice: &Slice) -> &[u8] {
        &self.data[slice.0..slice.1]
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "<HTTP Request {} {} ({:?})>",
               self.method(),
               self.path(),
               self.headers().into_iter().collect::<Vec<(&str, &str)>>())
    }
}

pub fn decode(buf: &mut BytesMut) -> io::Result<Option<Request>> {
    // TODO: we should grow this headers array if parsing fails and asks
    //       for more headers
    let (method, path, headers) = {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut r = httparse::Request::new(&mut headers);

        let status = try!(r.parse(buf)
                              .map_err(|e| {
                                           let msg = format!("failed to parse http request: {:?}",
                                                             e);
                                           io::Error::new(io::ErrorKind::Other, msg)
                                       }));

        match status {
            httparse::Status::Complete(amt) => amt,
            httparse::Status::Partial => return Ok(None),
        };

        let toslice = |a: &[u8]| {
            let start = a.as_ptr() as usize - buf.as_ptr() as usize;
            assert!(start < buf.len());
            (start, start + a.len())
        };

        (toslice(r.method.unwrap().as_bytes()),
         toslice(r.path.unwrap().as_bytes()),
         r.headers
             .iter()
             .map(|h| (toslice(h.name.as_bytes()), toslice(h.value)))
             .collect())
    };

    Ok(Request {
               method: method,
               path: path,
               headers: headers,
               data: buf.take(),
           }
           .into())
}

impl<'req> Iterator for RequestHeaders<'req> {
    type Item = (&'req str, &'req str);

    fn next(&mut self) -> Option<(&'req str, &'req str)> {
        self.headers
            .next()
            .map(|&(ref a, ref b)| {
                     let a = self.req.slice(a);
                     let b = self.req.slice(b);
                     (str::from_utf8(a).unwrap(), str::from_utf8(b).unwrap())
                 })
    }
}
