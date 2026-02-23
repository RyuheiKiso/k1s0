import Testing
@testable import K1s0SchemaRegistry

@Suite("SchemaRegistry Tests")
struct SchemaRegistryTests {
    @Test("subjectName が Confluent 規則に従うこと")
    func testSubjectName() {
        let name = SchemaRegistryConfig.subjectName(for: "k1s0.service.orders.order-created.v1")
        #expect(name == "k1s0.service.orders.order-created.v1-value")
    }

    @Test("SchemaType の rawValue が正しいこと")
    func testSchemaTypeRawValue() {
        #expect(SchemaType.avro.rawValue == "AVRO")
        #expect(SchemaType.json.rawValue == "JSON")
        #expect(SchemaType.protobuf.rawValue == "PROTOBUF")
    }

    @Test("CompatibilityMode の rawValue が正しいこと")
    func testCompatibilityModeRawValue() {
        #expect(CompatibilityMode.backward.rawValue == "BACKWARD")
        #expect(CompatibilityMode.full.rawValue == "FULL")
        #expect(CompatibilityMode.none.rawValue == "NONE")
    }

    @Test("SchemaRegistryError の説明が含まれること")
    func testSchemaRegistryErrorDescription() {
        let error = SchemaRegistryError.schemaNotFound(subject: "test-value", version: 1)
        #expect(error.description.contains("SCHEMA_NOT_FOUND"))
        #expect(error.description.contains("test-value"))
    }
}
