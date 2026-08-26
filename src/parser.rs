use nom::{
    bytes::complete::tag,
    multi::{count, length_data, many1},
    number::complete::{be_f32, be_i16, be_i32, be_i64, be_i8, be_u32},
    IResult,
};

use crate::{Centroid, Frame, Frames, OpdFile, OpdHeader};

pub fn parse(input: &[u8]) -> IResult<&[u8], OpdFile> {
    let (input, _) = tag(b".opd".as_slice())(input)?;

    let (input, json_header) = length_data(be_u32)(input)?;
    let header: crate::OpdHeader = serde_json::from_slice(json_header).unwrap();

    let (mut input, centroids) =
        count(parse_centroid, header.directive.num_centroids.unwrap())(input)?;

    let _base_offset = header.directive.num_centroids.unwrap() * 4 * 4;
    let _frame_data_len = header.directive.precision * 3;

    let frames = match header.directive.precision {
        1 => {
            let (next_input, frames) = parse_frame(input, &header, be_i8)?;
            input = next_input;
            Frames::I8(frames)
        }
        2 => {
            let (next_input, frames) = parse_frame(input, &header, be_i16)?;
            input = next_input;
            Frames::I16(frames)
        }
        4 => {
            let (next_input, frames) = parse_frame(input, &header, be_i32)?;
            input = next_input;
            Frames::I32(frames)
        }
        8 => {
            let (next_input, frames) = parse_frame(input, &header, be_i64)?;
            input = next_input;
            Frames::I64(frames)
        }
        _ => {
            unimplemented!()
        }
    };

    Ok((
        input,
        OpdFile {
            header,
            centroids,
            frames,
        },
    ))
}

type NumberParser<'a, NUM> = fn(input: &'a [u8]) -> IResult<&'a [u8], NUM>;

pub fn parse_frame<'a, T>(
    mut input: &'a [u8],
    header: &OpdHeader,
    number_parser: NumberParser<'a, T>,
) -> IResult<&'a [u8], Vec<Frame<T>>> {
    assert_eq!(header.directive.precision, std::mem::size_of::<T>());
    let base_offset = header.directive.num_centroids.unwrap() * 4 * 4;

    let mut frames = Vec::with_capacity(header.directive.frames.len());
    for frame in header.directive.frames.windows(2) {
        let start = (frame[0].offset - base_offset) / header.directive.precision;
        let end = (frame[1].offset - base_offset) / header.directive.precision;
        let len = end - start;

        let (new_input, data) = count(number_parser, len)(input)?;
        input = new_input;
        frames.push(Frame {
            time: frame[0].time,
            data,
        });
    }
    if let Some(last_frame) = header.directive.frames.last() {
        let (rest, data) = many1(number_parser)(input)?;
        frames.push(Frame {
            time: last_frame.time,
            data,
        });
        Ok((rest, frames))
    } else {
        Ok((input, frames))
    }
}

pub fn parse_centroid(input: &[u8]) -> IResult<&[u8], Centroid> {
    let (input, parent_id) = be_u32(input)?;
    let (input, x) = be_f32(input)?;
    let (input, y) = be_f32(input)?;
    let (input, z) = be_f32(input)?;
    Ok((
        input,
        Centroid {
            parent_id,
            offset: [x, y, z],
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::{parse, Frames};

    /// One centroid, i8 precision, two frames of three components each. Frame
    /// offsets are absolute file offsets past the centroid block, which is why
    /// they start at `num_centroids * 4 * 4` rather than at zero.
    fn minimal_opd() -> Vec<u8> {
        let header = br#"{
            "version": "1.0",
            "type": "opd",
            "directive": {
                "version": "1.0",
                "meta": { "projectId": "p", "projectName": "n" },
                "numCentroids": 1,
                "origin": { "x": 1.0, "y": 2.0, "z": 3.0 },
                "precision": 1,
                "scale": [1.0, 1.0, 1.0],
                "frames": [
                    { "time": 0.0, "offset": 16 },
                    { "time": 1.0, "offset": 19 }
                ]
            }
        }"#;

        let mut out = b".opd".to_vec();
        out.extend((header.len() as u32).to_be_bytes());
        out.extend(header);
        out.extend(7u32.to_be_bytes()); // centroid parent_id
        out.extend(0.5f32.to_be_bytes());
        out.extend(1.5f32.to_be_bytes());
        out.extend(2.5f32.to_be_bytes());
        out.extend([127i8, 0, 0].map(|v| v as u8)); // frame 0
        out.extend([0i8, 127, 0].map(|v| v as u8)); // frame 1
        out
    }

    #[test]
    fn parses_header_centroids_and_frames() {
        let bytes = minimal_opd();
        let (rest, file) = parse(&bytes).expect("parse failed");

        assert!(rest.is_empty(), "unconsumed trailing bytes: {rest:?}");
        assert_eq!(file.header.directive.origin.x, 1.0);
        assert_eq!(file.centroids.len(), 1);
        assert_eq!(file.centroids[0].parent_id, 7);
        assert_eq!(file.centroids[0].offset, [0.5, 1.5, 2.5]);

        let Frames::I8(frames) = file.frames else {
            panic!("precision 1 must yield Frames::I8");
        };
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].time, 0.0);
        assert_eq!(frames[1].time, 1.0);

        // i8 components are normalized against i8::MAX by the frame iterator.
        let decoded: Vec<[f32; 3]> = frames[0].into_iter().collect();
        assert_eq!(decoded, vec![[1.0, 0.0, 0.0]]);
    }
}
