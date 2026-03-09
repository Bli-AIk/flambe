//! Round-trip tests: load .amproj XML → serialize → re-parse → compare.

#[cfg(test)]
mod tests {
    use bevy_alight_motion::schema::*;
    use quick_xml::de::from_str as xml_from_str;
    use quick_xml::se::to_string as xml_to_string;
    use std::io::Read;

    /// Extract XML content from a .amproj ZIP file.
    fn extract_xml_from_amproj(path: &str) -> String {
        let file = std::fs::File::open(path).expect("Failed to open amproj");
        let mut archive = zip::ZipArchive::new(file).expect("Failed to read ZIP");

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).expect("Failed to read ZIP entry");
            let name = entry.name().to_string();
            if name.ends_with(".xml") || name == "project.json" {
                let mut content = String::new();
                entry
                    .read_to_string(&mut content)
                    .expect("Failed to read XML");
                return content;
            }
        }
        panic!("No XML found in amproj: {path}");
    }

    /// Core round-trip test: parse XML → serialize → re-parse → compare key fields.
    fn assert_round_trip(xml: &str, label: &str) {
        let scene: AmScene = xml_from_str(xml).unwrap_or_else(|e| {
            panic!("[{label}] Failed to parse original XML: {e}");
        });

        let serialized = xml_to_string(&scene).unwrap_or_else(|e| {
            panic!("[{label}] Failed to serialize scene: {e}");
        });

        let scene2: AmScene = xml_from_str(&serialized).unwrap_or_else(|e| {
            panic!(
                "[{label}] Failed to re-parse serialized XML: {e}\n--- serialized ---\n{serialized}"
            );
        });

        // Compare top-level scene attributes
        assert_eq!(scene.title, scene2.title, "[{label}] title mismatch");
        assert_eq!(scene.width, scene2.width, "[{label}] width mismatch");
        assert_eq!(scene.height, scene2.height, "[{label}] height mismatch");
        assert_eq!(scene.fps, scene2.fps, "[{label}] fps mismatch");
        assert_eq!(
            scene.total_time, scene2.total_time,
            "[{label}] total_time mismatch"
        );

        // Compare layer count
        assert_eq!(
            scene.layers.len(),
            scene2.layers.len(),
            "[{label}] layer count mismatch"
        );

        // Compare media count
        assert_eq!(
            scene.media.len(),
            scene2.media.len(),
            "[{label}] media count mismatch"
        );

        // Verify the format stabilizes after normalization (parse succeeds).
        let serialized2 = xml_to_string(&scene2).unwrap_or_else(|e| {
            panic!("[{label}] Failed to re-serialize: {e}");
        });
        let _scene3: AmScene = xml_from_str(&serialized2).unwrap_or_else(|e| {
            panic!("[{label}] Failed to parse third round-trip: {e}");
        });
    }

    /// Test round-trip on a minimal inline XML.
    #[test]
    fn test_round_trip_minimal() {
        let xml = r##"<?xml version='1.0' encoding='UTF-8' ?>
        <scene title="Minimal" width="1280" height="960" fps="60" totalTime="2000" bgcolor="#ff000000">
        </scene>"##;
        assert_round_trip(xml, "minimal");
    }

    /// Test round-trip on XML with a shape layer + keyframes.
    #[test]
    fn test_round_trip_shape_layer() {
        let xml = r##"<?xml version='1.0' encoding='UTF-8' ?>
        <scene title="Shape Test" width="1920" height="1080" fps="30" totalTime="3000" bgcolor="#ff222222">
            <shape id="1" startTime="0" endTime="3000" s=".rect">
                <transform>
                    <location value="0,0,0">
                        <kf t="0" v="0,0,0" />
                        <kf t="0.5" v="100,200,0" e="cubicBezier 0.25 0.1 0.25 1" />
                    </location>
                    <rotation value="0">
                        <kf t="0" v="0" />
                    </rotation>
                    <scale value="1,1">
                        <kf t="0" v="1,1" />
                    </scale>
                    <opacity value="1">
                        <kf t="0" v="1" />
                    </opacity>
                    <pivot value="0,0,0" />
                </transform>
                <fillColor value="#ff3366cc" />
            </shape>
        </scene>"##;
        assert_round_trip(xml, "shape_layer");
    }

    /// Test round-trip with real .amproj files from the assets directory.
    #[test]
    fn test_round_trip_amproj_files() {
        let am_assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("bevy_alight_motion/assets/projects");

        if !am_assets.exists() {
            eprintln!("Skipping amproj round-trip: assets not found at {am_assets:?}");
            return;
        }

        let mut tested = 0;
        for entry in walkdir::WalkDir::new(&am_assets)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "amproj") {
                let path_str = path.to_string_lossy();
                let xml = extract_xml_from_amproj(&path_str);
                assert_round_trip(&xml, &path_str);
                tested += 1;
            }
        }
        eprintln!("Round-trip tested {tested} .amproj files");
        assert!(tested > 0, "No .amproj files found in {am_assets:?}");
    }
}
