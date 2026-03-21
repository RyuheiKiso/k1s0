// This is a generated file - do not edit.
//
// Generated from k1s0/system/mastermaintenance/v1/master_maintenance.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:async' as $async;
import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'package:protobuf/protobuf.dart' as $pb;

import 'master_maintenance.pb.dart' as $0;

export 'master_maintenance.pb.dart';

@$pb.GrpcServiceName(
    'k1s0.system.mastermaintenance.v1.MasterMaintenanceService')
class MasterMaintenanceServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  MasterMaintenanceServiceClient(super.channel,
      {super.options, super.interceptors});

  /// テーブル定義
  $grpc.ResponseFuture<$0.CreateTableDefinitionResponse> createTableDefinition(
    $0.CreateTableDefinitionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createTableDefinition, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateTableDefinitionResponse> updateTableDefinition(
    $0.UpdateTableDefinitionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateTableDefinition, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteTableDefinitionResponse> deleteTableDefinition(
    $0.DeleteTableDefinitionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteTableDefinition, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetTableDefinitionResponse> getTableDefinition(
    $0.GetTableDefinitionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getTableDefinition, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListTableDefinitionsResponse> listTableDefinitions(
    $0.ListTableDefinitionsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listTableDefinitions, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListColumnsResponse> listColumns(
    $0.ListColumnsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listColumns, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateColumnsResponse> createColumns(
    $0.CreateColumnsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createColumns, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateColumnResponse> updateColumn(
    $0.UpdateColumnRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateColumn, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteColumnResponse> deleteColumn(
    $0.DeleteColumnRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteColumn, request, options: options);
  }

  /// データ CRUD
  $grpc.ResponseFuture<$0.GetRecordResponse> getRecord(
    $0.GetRecordRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getRecord, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListRecordsResponse> listRecords(
    $0.ListRecordsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listRecords, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateRecordResponse> createRecord(
    $0.CreateRecordRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createRecord, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateRecordResponse> updateRecord(
    $0.UpdateRecordRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateRecord, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteRecordResponse> deleteRecord(
    $0.DeleteRecordRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteRecord, request, options: options);
  }

  /// 整合性チェック
  $grpc.ResponseFuture<$0.CheckConsistencyResponse> checkConsistency(
    $0.CheckConsistencyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$checkConsistency, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateRuleResponse> createRule(
    $0.CreateRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createRule, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetRuleResponse> getRule(
    $0.GetRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getRule, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateRuleResponse> updateRule(
    $0.UpdateRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateRule, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteRuleResponse> deleteRule(
    $0.DeleteRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteRule, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListRulesResponse> listRules(
    $0.ListRulesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listRules, request, options: options);
  }

  $grpc.ResponseFuture<$0.ExecuteRuleResponse> executeRule(
    $0.ExecuteRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$executeRule, request, options: options);
  }

  /// JSON Schema
  $grpc.ResponseFuture<$0.GetTableSchemaResponse> getTableSchema(
    $0.GetTableSchemaRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getTableSchema, request, options: options);
  }

  /// Relationship
  $grpc.ResponseFuture<$0.ListRelationshipsResponse> listRelationships(
    $0.ListRelationshipsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listRelationships, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateRelationshipResponse> createRelationship(
    $0.CreateRelationshipRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createRelationship, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateRelationshipResponse> updateRelationship(
    $0.UpdateRelationshipRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateRelationship, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteRelationshipResponse> deleteRelationship(
    $0.DeleteRelationshipRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteRelationship, request, options: options);
  }

  /// Import / Export
  $grpc.ResponseFuture<$0.ImportRecordsResponse> importRecords(
    $0.ImportRecordsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$importRecords, request, options: options);
  }

  $grpc.ResponseFuture<$0.ExportRecordsResponse> exportRecords(
    $0.ExportRecordsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$exportRecords, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetImportJobResponse> getImportJob(
    $0.GetImportJobRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getImportJob, request, options: options);
  }

  /// Display Config
  $grpc.ResponseFuture<$0.ListDisplayConfigsResponse> listDisplayConfigs(
    $0.ListDisplayConfigsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listDisplayConfigs, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetDisplayConfigResponse> getDisplayConfig(
    $0.GetDisplayConfigRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getDisplayConfig, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateDisplayConfigResponse> createDisplayConfig(
    $0.CreateDisplayConfigRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createDisplayConfig, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateDisplayConfigResponse> updateDisplayConfig(
    $0.UpdateDisplayConfigRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateDisplayConfig, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteDisplayConfigResponse> deleteDisplayConfig(
    $0.DeleteDisplayConfigRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteDisplayConfig, request, options: options);
  }

  /// Audit Logs
  $grpc.ResponseFuture<$0.ListTableAuditLogsResponse> listTableAuditLogs(
    $0.ListTableAuditLogsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listTableAuditLogs, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListRecordAuditLogsResponse> listRecordAuditLogs(
    $0.ListRecordAuditLogsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listRecordAuditLogs, request, options: options);
  }

  /// Domain Management
  $grpc.ResponseFuture<$0.ListDomainsResponse> listDomains(
    $0.ListDomainsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listDomains, request, options: options);
  }

  // method descriptors

  static final _$createTableDefinition = $grpc.ClientMethod<
          $0.CreateTableDefinitionRequest, $0.CreateTableDefinitionResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateTableDefinition',
      ($0.CreateTableDefinitionRequest value) => value.writeToBuffer(),
      $0.CreateTableDefinitionResponse.fromBuffer);
  static final _$updateTableDefinition = $grpc.ClientMethod<
          $0.UpdateTableDefinitionRequest, $0.UpdateTableDefinitionResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateTableDefinition',
      ($0.UpdateTableDefinitionRequest value) => value.writeToBuffer(),
      $0.UpdateTableDefinitionResponse.fromBuffer);
  static final _$deleteTableDefinition = $grpc.ClientMethod<
          $0.DeleteTableDefinitionRequest, $0.DeleteTableDefinitionResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteTableDefinition',
      ($0.DeleteTableDefinitionRequest value) => value.writeToBuffer(),
      $0.DeleteTableDefinitionResponse.fromBuffer);
  static final _$getTableDefinition = $grpc.ClientMethod<
          $0.GetTableDefinitionRequest, $0.GetTableDefinitionResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetTableDefinition',
      ($0.GetTableDefinitionRequest value) => value.writeToBuffer(),
      $0.GetTableDefinitionResponse.fromBuffer);
  static final _$listTableDefinitions = $grpc.ClientMethod<
          $0.ListTableDefinitionsRequest, $0.ListTableDefinitionsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListTableDefinitions',
      ($0.ListTableDefinitionsRequest value) => value.writeToBuffer(),
      $0.ListTableDefinitionsResponse.fromBuffer);
  static final _$listColumns = $grpc.ClientMethod<$0.ListColumnsRequest,
          $0.ListColumnsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListColumns',
      ($0.ListColumnsRequest value) => value.writeToBuffer(),
      $0.ListColumnsResponse.fromBuffer);
  static final _$createColumns = $grpc.ClientMethod<$0.CreateColumnsRequest,
          $0.CreateColumnsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateColumns',
      ($0.CreateColumnsRequest value) => value.writeToBuffer(),
      $0.CreateColumnsResponse.fromBuffer);
  static final _$updateColumn = $grpc.ClientMethod<$0.UpdateColumnRequest,
          $0.UpdateColumnResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateColumn',
      ($0.UpdateColumnRequest value) => value.writeToBuffer(),
      $0.UpdateColumnResponse.fromBuffer);
  static final _$deleteColumn = $grpc.ClientMethod<$0.DeleteColumnRequest,
          $0.DeleteColumnResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteColumn',
      ($0.DeleteColumnRequest value) => value.writeToBuffer(),
      $0.DeleteColumnResponse.fromBuffer);
  static final _$getRecord = $grpc.ClientMethod<$0.GetRecordRequest,
          $0.GetRecordResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetRecord',
      ($0.GetRecordRequest value) => value.writeToBuffer(),
      $0.GetRecordResponse.fromBuffer);
  static final _$listRecords = $grpc.ClientMethod<$0.ListRecordsRequest,
          $0.ListRecordsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRecords',
      ($0.ListRecordsRequest value) => value.writeToBuffer(),
      $0.ListRecordsResponse.fromBuffer);
  static final _$createRecord = $grpc.ClientMethod<$0.CreateRecordRequest,
          $0.CreateRecordResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateRecord',
      ($0.CreateRecordRequest value) => value.writeToBuffer(),
      $0.CreateRecordResponse.fromBuffer);
  static final _$updateRecord = $grpc.ClientMethod<$0.UpdateRecordRequest,
          $0.UpdateRecordResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateRecord',
      ($0.UpdateRecordRequest value) => value.writeToBuffer(),
      $0.UpdateRecordResponse.fromBuffer);
  static final _$deleteRecord = $grpc.ClientMethod<$0.DeleteRecordRequest,
          $0.DeleteRecordResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteRecord',
      ($0.DeleteRecordRequest value) => value.writeToBuffer(),
      $0.DeleteRecordResponse.fromBuffer);
  static final _$checkConsistency = $grpc.ClientMethod<
          $0.CheckConsistencyRequest, $0.CheckConsistencyResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CheckConsistency',
      ($0.CheckConsistencyRequest value) => value.writeToBuffer(),
      $0.CheckConsistencyResponse.fromBuffer);
  static final _$createRule = $grpc.ClientMethod<$0.CreateRuleRequest,
          $0.CreateRuleResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateRule',
      ($0.CreateRuleRequest value) => value.writeToBuffer(),
      $0.CreateRuleResponse.fromBuffer);
  static final _$getRule =
      $grpc.ClientMethod<$0.GetRuleRequest, $0.GetRuleResponse>(
          '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetRule',
          ($0.GetRuleRequest value) => value.writeToBuffer(),
          $0.GetRuleResponse.fromBuffer);
  static final _$updateRule = $grpc.ClientMethod<$0.UpdateRuleRequest,
          $0.UpdateRuleResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateRule',
      ($0.UpdateRuleRequest value) => value.writeToBuffer(),
      $0.UpdateRuleResponse.fromBuffer);
  static final _$deleteRule = $grpc.ClientMethod<$0.DeleteRuleRequest,
          $0.DeleteRuleResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteRule',
      ($0.DeleteRuleRequest value) => value.writeToBuffer(),
      $0.DeleteRuleResponse.fromBuffer);
  static final _$listRules = $grpc.ClientMethod<$0.ListRulesRequest,
          $0.ListRulesResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRules',
      ($0.ListRulesRequest value) => value.writeToBuffer(),
      $0.ListRulesResponse.fromBuffer);
  static final _$executeRule = $grpc.ClientMethod<$0.ExecuteRuleRequest,
          $0.ExecuteRuleResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ExecuteRule',
      ($0.ExecuteRuleRequest value) => value.writeToBuffer(),
      $0.ExecuteRuleResponse.fromBuffer);
  static final _$getTableSchema = $grpc.ClientMethod<$0.GetTableSchemaRequest,
          $0.GetTableSchemaResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetTableSchema',
      ($0.GetTableSchemaRequest value) => value.writeToBuffer(),
      $0.GetTableSchemaResponse.fromBuffer);
  static final _$listRelationships = $grpc.ClientMethod<
          $0.ListRelationshipsRequest, $0.ListRelationshipsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRelationships',
      ($0.ListRelationshipsRequest value) => value.writeToBuffer(),
      $0.ListRelationshipsResponse.fromBuffer);
  static final _$createRelationship = $grpc.ClientMethod<
          $0.CreateRelationshipRequest, $0.CreateRelationshipResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateRelationship',
      ($0.CreateRelationshipRequest value) => value.writeToBuffer(),
      $0.CreateRelationshipResponse.fromBuffer);
  static final _$updateRelationship = $grpc.ClientMethod<
          $0.UpdateRelationshipRequest, $0.UpdateRelationshipResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateRelationship',
      ($0.UpdateRelationshipRequest value) => value.writeToBuffer(),
      $0.UpdateRelationshipResponse.fromBuffer);
  static final _$deleteRelationship = $grpc.ClientMethod<
          $0.DeleteRelationshipRequest, $0.DeleteRelationshipResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteRelationship',
      ($0.DeleteRelationshipRequest value) => value.writeToBuffer(),
      $0.DeleteRelationshipResponse.fromBuffer);
  static final _$importRecords = $grpc.ClientMethod<$0.ImportRecordsRequest,
          $0.ImportRecordsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ImportRecords',
      ($0.ImportRecordsRequest value) => value.writeToBuffer(),
      $0.ImportRecordsResponse.fromBuffer);
  static final _$exportRecords = $grpc.ClientMethod<$0.ExportRecordsRequest,
          $0.ExportRecordsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ExportRecords',
      ($0.ExportRecordsRequest value) => value.writeToBuffer(),
      $0.ExportRecordsResponse.fromBuffer);
  static final _$getImportJob = $grpc.ClientMethod<$0.GetImportJobRequest,
          $0.GetImportJobResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetImportJob',
      ($0.GetImportJobRequest value) => value.writeToBuffer(),
      $0.GetImportJobResponse.fromBuffer);
  static final _$listDisplayConfigs = $grpc.ClientMethod<
          $0.ListDisplayConfigsRequest, $0.ListDisplayConfigsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListDisplayConfigs',
      ($0.ListDisplayConfigsRequest value) => value.writeToBuffer(),
      $0.ListDisplayConfigsResponse.fromBuffer);
  static final _$getDisplayConfig = $grpc.ClientMethod<
          $0.GetDisplayConfigRequest, $0.GetDisplayConfigResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetDisplayConfig',
      ($0.GetDisplayConfigRequest value) => value.writeToBuffer(),
      $0.GetDisplayConfigResponse.fromBuffer);
  static final _$createDisplayConfig = $grpc.ClientMethod<
          $0.CreateDisplayConfigRequest, $0.CreateDisplayConfigResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateDisplayConfig',
      ($0.CreateDisplayConfigRequest value) => value.writeToBuffer(),
      $0.CreateDisplayConfigResponse.fromBuffer);
  static final _$updateDisplayConfig = $grpc.ClientMethod<
          $0.UpdateDisplayConfigRequest, $0.UpdateDisplayConfigResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateDisplayConfig',
      ($0.UpdateDisplayConfigRequest value) => value.writeToBuffer(),
      $0.UpdateDisplayConfigResponse.fromBuffer);
  static final _$deleteDisplayConfig = $grpc.ClientMethod<
          $0.DeleteDisplayConfigRequest, $0.DeleteDisplayConfigResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteDisplayConfig',
      ($0.DeleteDisplayConfigRequest value) => value.writeToBuffer(),
      $0.DeleteDisplayConfigResponse.fromBuffer);
  static final _$listTableAuditLogs = $grpc.ClientMethod<
          $0.ListTableAuditLogsRequest, $0.ListTableAuditLogsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListTableAuditLogs',
      ($0.ListTableAuditLogsRequest value) => value.writeToBuffer(),
      $0.ListTableAuditLogsResponse.fromBuffer);
  static final _$listRecordAuditLogs = $grpc.ClientMethod<
          $0.ListRecordAuditLogsRequest, $0.ListRecordAuditLogsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRecordAuditLogs',
      ($0.ListRecordAuditLogsRequest value) => value.writeToBuffer(),
      $0.ListRecordAuditLogsResponse.fromBuffer);
  static final _$listDomains = $grpc.ClientMethod<$0.ListDomainsRequest,
          $0.ListDomainsResponse>(
      '/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListDomains',
      ($0.ListDomainsRequest value) => value.writeToBuffer(),
      $0.ListDomainsResponse.fromBuffer);
}

@$pb.GrpcServiceName(
    'k1s0.system.mastermaintenance.v1.MasterMaintenanceService')
abstract class MasterMaintenanceServiceBase extends $grpc.Service {
  $core.String get $name =>
      'k1s0.system.mastermaintenance.v1.MasterMaintenanceService';

  MasterMaintenanceServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.CreateTableDefinitionRequest,
            $0.CreateTableDefinitionResponse>(
        'CreateTableDefinition',
        createTableDefinition_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CreateTableDefinitionRequest.fromBuffer(value),
        ($0.CreateTableDefinitionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateTableDefinitionRequest,
            $0.UpdateTableDefinitionResponse>(
        'UpdateTableDefinition',
        updateTableDefinition_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdateTableDefinitionRequest.fromBuffer(value),
        ($0.UpdateTableDefinitionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteTableDefinitionRequest,
            $0.DeleteTableDefinitionResponse>(
        'DeleteTableDefinition',
        deleteTableDefinition_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteTableDefinitionRequest.fromBuffer(value),
        ($0.DeleteTableDefinitionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetTableDefinitionRequest,
            $0.GetTableDefinitionResponse>(
        'GetTableDefinition',
        getTableDefinition_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetTableDefinitionRequest.fromBuffer(value),
        ($0.GetTableDefinitionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListTableDefinitionsRequest,
            $0.ListTableDefinitionsResponse>(
        'ListTableDefinitions',
        listTableDefinitions_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListTableDefinitionsRequest.fromBuffer(value),
        ($0.ListTableDefinitionsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListColumnsRequest, $0.ListColumnsResponse>(
            'ListColumns',
            listColumns_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListColumnsRequest.fromBuffer(value),
            ($0.ListColumnsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.CreateColumnsRequest, $0.CreateColumnsResponse>(
            'CreateColumns',
            createColumns_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateColumnsRequest.fromBuffer(value),
            ($0.CreateColumnsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpdateColumnRequest, $0.UpdateColumnResponse>(
            'UpdateColumn',
            updateColumn_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpdateColumnRequest.fromBuffer(value),
            ($0.UpdateColumnResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteColumnRequest, $0.DeleteColumnResponse>(
            'DeleteColumn',
            deleteColumn_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteColumnRequest.fromBuffer(value),
            ($0.DeleteColumnResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetRecordRequest, $0.GetRecordResponse>(
        'GetRecord',
        getRecord_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetRecordRequest.fromBuffer(value),
        ($0.GetRecordResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListRecordsRequest, $0.ListRecordsResponse>(
            'ListRecords',
            listRecords_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListRecordsRequest.fromBuffer(value),
            ($0.ListRecordsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.CreateRecordRequest, $0.CreateRecordResponse>(
            'CreateRecord',
            createRecord_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateRecordRequest.fromBuffer(value),
            ($0.CreateRecordResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpdateRecordRequest, $0.UpdateRecordResponse>(
            'UpdateRecord',
            updateRecord_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpdateRecordRequest.fromBuffer(value),
            ($0.UpdateRecordResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteRecordRequest, $0.DeleteRecordResponse>(
            'DeleteRecord',
            deleteRecord_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteRecordRequest.fromBuffer(value),
            ($0.DeleteRecordResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CheckConsistencyRequest,
            $0.CheckConsistencyResponse>(
        'CheckConsistency',
        checkConsistency_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CheckConsistencyRequest.fromBuffer(value),
        ($0.CheckConsistencyResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateRuleRequest, $0.CreateRuleResponse>(
        'CreateRule',
        createRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CreateRuleRequest.fromBuffer(value),
        ($0.CreateRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetRuleRequest, $0.GetRuleResponse>(
        'GetRule',
        getRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetRuleRequest.fromBuffer(value),
        ($0.GetRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateRuleRequest, $0.UpdateRuleResponse>(
        'UpdateRule',
        updateRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.UpdateRuleRequest.fromBuffer(value),
        ($0.UpdateRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteRuleRequest, $0.DeleteRuleResponse>(
        'DeleteRule',
        deleteRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.DeleteRuleRequest.fromBuffer(value),
        ($0.DeleteRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListRulesRequest, $0.ListRulesResponse>(
        'ListRules',
        listRules_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListRulesRequest.fromBuffer(value),
        ($0.ListRulesResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ExecuteRuleRequest, $0.ExecuteRuleResponse>(
            'ExecuteRule',
            executeRule_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ExecuteRuleRequest.fromBuffer(value),
            ($0.ExecuteRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetTableSchemaRequest,
            $0.GetTableSchemaResponse>(
        'GetTableSchema',
        getTableSchema_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetTableSchemaRequest.fromBuffer(value),
        ($0.GetTableSchemaResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListRelationshipsRequest,
            $0.ListRelationshipsResponse>(
        'ListRelationships',
        listRelationships_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListRelationshipsRequest.fromBuffer(value),
        ($0.ListRelationshipsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateRelationshipRequest,
            $0.CreateRelationshipResponse>(
        'CreateRelationship',
        createRelationship_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CreateRelationshipRequest.fromBuffer(value),
        ($0.CreateRelationshipResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateRelationshipRequest,
            $0.UpdateRelationshipResponse>(
        'UpdateRelationship',
        updateRelationship_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdateRelationshipRequest.fromBuffer(value),
        ($0.UpdateRelationshipResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteRelationshipRequest,
            $0.DeleteRelationshipResponse>(
        'DeleteRelationship',
        deleteRelationship_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteRelationshipRequest.fromBuffer(value),
        ($0.DeleteRelationshipResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ImportRecordsRequest, $0.ImportRecordsResponse>(
            'ImportRecords',
            importRecords_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ImportRecordsRequest.fromBuffer(value),
            ($0.ImportRecordsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ExportRecordsRequest, $0.ExportRecordsResponse>(
            'ExportRecords',
            exportRecords_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ExportRecordsRequest.fromBuffer(value),
            ($0.ExportRecordsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetImportJobRequest, $0.GetImportJobResponse>(
            'GetImportJob',
            getImportJob_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetImportJobRequest.fromBuffer(value),
            ($0.GetImportJobResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListDisplayConfigsRequest,
            $0.ListDisplayConfigsResponse>(
        'ListDisplayConfigs',
        listDisplayConfigs_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListDisplayConfigsRequest.fromBuffer(value),
        ($0.ListDisplayConfigsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetDisplayConfigRequest,
            $0.GetDisplayConfigResponse>(
        'GetDisplayConfig',
        getDisplayConfig_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetDisplayConfigRequest.fromBuffer(value),
        ($0.GetDisplayConfigResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateDisplayConfigRequest,
            $0.CreateDisplayConfigResponse>(
        'CreateDisplayConfig',
        createDisplayConfig_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CreateDisplayConfigRequest.fromBuffer(value),
        ($0.CreateDisplayConfigResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateDisplayConfigRequest,
            $0.UpdateDisplayConfigResponse>(
        'UpdateDisplayConfig',
        updateDisplayConfig_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdateDisplayConfigRequest.fromBuffer(value),
        ($0.UpdateDisplayConfigResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteDisplayConfigRequest,
            $0.DeleteDisplayConfigResponse>(
        'DeleteDisplayConfig',
        deleteDisplayConfig_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteDisplayConfigRequest.fromBuffer(value),
        ($0.DeleteDisplayConfigResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListTableAuditLogsRequest,
            $0.ListTableAuditLogsResponse>(
        'ListTableAuditLogs',
        listTableAuditLogs_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListTableAuditLogsRequest.fromBuffer(value),
        ($0.ListTableAuditLogsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListRecordAuditLogsRequest,
            $0.ListRecordAuditLogsResponse>(
        'ListRecordAuditLogs',
        listRecordAuditLogs_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListRecordAuditLogsRequest.fromBuffer(value),
        ($0.ListRecordAuditLogsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListDomainsRequest, $0.ListDomainsResponse>(
            'ListDomains',
            listDomains_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListDomainsRequest.fromBuffer(value),
            ($0.ListDomainsResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.CreateTableDefinitionResponse> createTableDefinition_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateTableDefinitionRequest> $request) async {
    return createTableDefinition($call, await $request);
  }

  $async.Future<$0.CreateTableDefinitionResponse> createTableDefinition(
      $grpc.ServiceCall call, $0.CreateTableDefinitionRequest request);

  $async.Future<$0.UpdateTableDefinitionResponse> updateTableDefinition_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateTableDefinitionRequest> $request) async {
    return updateTableDefinition($call, await $request);
  }

  $async.Future<$0.UpdateTableDefinitionResponse> updateTableDefinition(
      $grpc.ServiceCall call, $0.UpdateTableDefinitionRequest request);

  $async.Future<$0.DeleteTableDefinitionResponse> deleteTableDefinition_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteTableDefinitionRequest> $request) async {
    return deleteTableDefinition($call, await $request);
  }

  $async.Future<$0.DeleteTableDefinitionResponse> deleteTableDefinition(
      $grpc.ServiceCall call, $0.DeleteTableDefinitionRequest request);

  $async.Future<$0.GetTableDefinitionResponse> getTableDefinition_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetTableDefinitionRequest> $request) async {
    return getTableDefinition($call, await $request);
  }

  $async.Future<$0.GetTableDefinitionResponse> getTableDefinition(
      $grpc.ServiceCall call, $0.GetTableDefinitionRequest request);

  $async.Future<$0.ListTableDefinitionsResponse> listTableDefinitions_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListTableDefinitionsRequest> $request) async {
    return listTableDefinitions($call, await $request);
  }

  $async.Future<$0.ListTableDefinitionsResponse> listTableDefinitions(
      $grpc.ServiceCall call, $0.ListTableDefinitionsRequest request);

  $async.Future<$0.ListColumnsResponse> listColumns_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListColumnsRequest> $request) async {
    return listColumns($call, await $request);
  }

  $async.Future<$0.ListColumnsResponse> listColumns(
      $grpc.ServiceCall call, $0.ListColumnsRequest request);

  $async.Future<$0.CreateColumnsResponse> createColumns_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateColumnsRequest> $request) async {
    return createColumns($call, await $request);
  }

  $async.Future<$0.CreateColumnsResponse> createColumns(
      $grpc.ServiceCall call, $0.CreateColumnsRequest request);

  $async.Future<$0.UpdateColumnResponse> updateColumn_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateColumnRequest> $request) async {
    return updateColumn($call, await $request);
  }

  $async.Future<$0.UpdateColumnResponse> updateColumn(
      $grpc.ServiceCall call, $0.UpdateColumnRequest request);

  $async.Future<$0.DeleteColumnResponse> deleteColumn_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteColumnRequest> $request) async {
    return deleteColumn($call, await $request);
  }

  $async.Future<$0.DeleteColumnResponse> deleteColumn(
      $grpc.ServiceCall call, $0.DeleteColumnRequest request);

  $async.Future<$0.GetRecordResponse> getRecord_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetRecordRequest> $request) async {
    return getRecord($call, await $request);
  }

  $async.Future<$0.GetRecordResponse> getRecord(
      $grpc.ServiceCall call, $0.GetRecordRequest request);

  $async.Future<$0.ListRecordsResponse> listRecords_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListRecordsRequest> $request) async {
    return listRecords($call, await $request);
  }

  $async.Future<$0.ListRecordsResponse> listRecords(
      $grpc.ServiceCall call, $0.ListRecordsRequest request);

  $async.Future<$0.CreateRecordResponse> createRecord_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateRecordRequest> $request) async {
    return createRecord($call, await $request);
  }

  $async.Future<$0.CreateRecordResponse> createRecord(
      $grpc.ServiceCall call, $0.CreateRecordRequest request);

  $async.Future<$0.UpdateRecordResponse> updateRecord_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateRecordRequest> $request) async {
    return updateRecord($call, await $request);
  }

  $async.Future<$0.UpdateRecordResponse> updateRecord(
      $grpc.ServiceCall call, $0.UpdateRecordRequest request);

  $async.Future<$0.DeleteRecordResponse> deleteRecord_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteRecordRequest> $request) async {
    return deleteRecord($call, await $request);
  }

  $async.Future<$0.DeleteRecordResponse> deleteRecord(
      $grpc.ServiceCall call, $0.DeleteRecordRequest request);

  $async.Future<$0.CheckConsistencyResponse> checkConsistency_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CheckConsistencyRequest> $request) async {
    return checkConsistency($call, await $request);
  }

  $async.Future<$0.CheckConsistencyResponse> checkConsistency(
      $grpc.ServiceCall call, $0.CheckConsistencyRequest request);

  $async.Future<$0.CreateRuleResponse> createRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateRuleRequest> $request) async {
    return createRule($call, await $request);
  }

  $async.Future<$0.CreateRuleResponse> createRule(
      $grpc.ServiceCall call, $0.CreateRuleRequest request);

  $async.Future<$0.GetRuleResponse> getRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetRuleRequest> $request) async {
    return getRule($call, await $request);
  }

  $async.Future<$0.GetRuleResponse> getRule(
      $grpc.ServiceCall call, $0.GetRuleRequest request);

  $async.Future<$0.UpdateRuleResponse> updateRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.UpdateRuleRequest> $request) async {
    return updateRule($call, await $request);
  }

  $async.Future<$0.UpdateRuleResponse> updateRule(
      $grpc.ServiceCall call, $0.UpdateRuleRequest request);

  $async.Future<$0.DeleteRuleResponse> deleteRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.DeleteRuleRequest> $request) async {
    return deleteRule($call, await $request);
  }

  $async.Future<$0.DeleteRuleResponse> deleteRule(
      $grpc.ServiceCall call, $0.DeleteRuleRequest request);

  $async.Future<$0.ListRulesResponse> listRules_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListRulesRequest> $request) async {
    return listRules($call, await $request);
  }

  $async.Future<$0.ListRulesResponse> listRules(
      $grpc.ServiceCall call, $0.ListRulesRequest request);

  $async.Future<$0.ExecuteRuleResponse> executeRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ExecuteRuleRequest> $request) async {
    return executeRule($call, await $request);
  }

  $async.Future<$0.ExecuteRuleResponse> executeRule(
      $grpc.ServiceCall call, $0.ExecuteRuleRequest request);

  $async.Future<$0.GetTableSchemaResponse> getTableSchema_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetTableSchemaRequest> $request) async {
    return getTableSchema($call, await $request);
  }

  $async.Future<$0.GetTableSchemaResponse> getTableSchema(
      $grpc.ServiceCall call, $0.GetTableSchemaRequest request);

  $async.Future<$0.ListRelationshipsResponse> listRelationships_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListRelationshipsRequest> $request) async {
    return listRelationships($call, await $request);
  }

  $async.Future<$0.ListRelationshipsResponse> listRelationships(
      $grpc.ServiceCall call, $0.ListRelationshipsRequest request);

  $async.Future<$0.CreateRelationshipResponse> createRelationship_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateRelationshipRequest> $request) async {
    return createRelationship($call, await $request);
  }

  $async.Future<$0.CreateRelationshipResponse> createRelationship(
      $grpc.ServiceCall call, $0.CreateRelationshipRequest request);

  $async.Future<$0.UpdateRelationshipResponse> updateRelationship_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateRelationshipRequest> $request) async {
    return updateRelationship($call, await $request);
  }

  $async.Future<$0.UpdateRelationshipResponse> updateRelationship(
      $grpc.ServiceCall call, $0.UpdateRelationshipRequest request);

  $async.Future<$0.DeleteRelationshipResponse> deleteRelationship_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteRelationshipRequest> $request) async {
    return deleteRelationship($call, await $request);
  }

  $async.Future<$0.DeleteRelationshipResponse> deleteRelationship(
      $grpc.ServiceCall call, $0.DeleteRelationshipRequest request);

  $async.Future<$0.ImportRecordsResponse> importRecords_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ImportRecordsRequest> $request) async {
    return importRecords($call, await $request);
  }

  $async.Future<$0.ImportRecordsResponse> importRecords(
      $grpc.ServiceCall call, $0.ImportRecordsRequest request);

  $async.Future<$0.ExportRecordsResponse> exportRecords_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ExportRecordsRequest> $request) async {
    return exportRecords($call, await $request);
  }

  $async.Future<$0.ExportRecordsResponse> exportRecords(
      $grpc.ServiceCall call, $0.ExportRecordsRequest request);

  $async.Future<$0.GetImportJobResponse> getImportJob_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetImportJobRequest> $request) async {
    return getImportJob($call, await $request);
  }

  $async.Future<$0.GetImportJobResponse> getImportJob(
      $grpc.ServiceCall call, $0.GetImportJobRequest request);

  $async.Future<$0.ListDisplayConfigsResponse> listDisplayConfigs_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListDisplayConfigsRequest> $request) async {
    return listDisplayConfigs($call, await $request);
  }

  $async.Future<$0.ListDisplayConfigsResponse> listDisplayConfigs(
      $grpc.ServiceCall call, $0.ListDisplayConfigsRequest request);

  $async.Future<$0.GetDisplayConfigResponse> getDisplayConfig_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetDisplayConfigRequest> $request) async {
    return getDisplayConfig($call, await $request);
  }

  $async.Future<$0.GetDisplayConfigResponse> getDisplayConfig(
      $grpc.ServiceCall call, $0.GetDisplayConfigRequest request);

  $async.Future<$0.CreateDisplayConfigResponse> createDisplayConfig_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateDisplayConfigRequest> $request) async {
    return createDisplayConfig($call, await $request);
  }

  $async.Future<$0.CreateDisplayConfigResponse> createDisplayConfig(
      $grpc.ServiceCall call, $0.CreateDisplayConfigRequest request);

  $async.Future<$0.UpdateDisplayConfigResponse> updateDisplayConfig_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateDisplayConfigRequest> $request) async {
    return updateDisplayConfig($call, await $request);
  }

  $async.Future<$0.UpdateDisplayConfigResponse> updateDisplayConfig(
      $grpc.ServiceCall call, $0.UpdateDisplayConfigRequest request);

  $async.Future<$0.DeleteDisplayConfigResponse> deleteDisplayConfig_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteDisplayConfigRequest> $request) async {
    return deleteDisplayConfig($call, await $request);
  }

  $async.Future<$0.DeleteDisplayConfigResponse> deleteDisplayConfig(
      $grpc.ServiceCall call, $0.DeleteDisplayConfigRequest request);

  $async.Future<$0.ListTableAuditLogsResponse> listTableAuditLogs_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListTableAuditLogsRequest> $request) async {
    return listTableAuditLogs($call, await $request);
  }

  $async.Future<$0.ListTableAuditLogsResponse> listTableAuditLogs(
      $grpc.ServiceCall call, $0.ListTableAuditLogsRequest request);

  $async.Future<$0.ListRecordAuditLogsResponse> listRecordAuditLogs_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListRecordAuditLogsRequest> $request) async {
    return listRecordAuditLogs($call, await $request);
  }

  $async.Future<$0.ListRecordAuditLogsResponse> listRecordAuditLogs(
      $grpc.ServiceCall call, $0.ListRecordAuditLogsRequest request);

  $async.Future<$0.ListDomainsResponse> listDomains_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListDomainsRequest> $request) async {
    return listDomains($call, await $request);
  }

  $async.Future<$0.ListDomainsResponse> listDomains(
      $grpc.ServiceCall call, $0.ListDomainsRequest request);
}
