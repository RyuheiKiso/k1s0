// This is a generated file - do not edit.
//
// Generated from k1s0/system/ruleengine/v1/rule_engine.proto.

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

import 'rule_engine.pb.dart' as $0;

export 'rule_engine.pb.dart';

@$pb.GrpcServiceName('k1s0.system.ruleengine.v1.RuleEngineService')
class RuleEngineServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  RuleEngineServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.ListRulesResponse> listRules(
    $0.ListRulesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listRules, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetRuleResponse> getRule(
    $0.GetRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getRule, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateRuleResponse> createRule(
    $0.CreateRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createRule, request, options: options);
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

  $grpc.ResponseFuture<$0.ListRuleSetsResponse> listRuleSets(
    $0.ListRuleSetsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listRuleSets, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetRuleSetResponse> getRuleSet(
    $0.GetRuleSetRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getRuleSet, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateRuleSetResponse> createRuleSet(
    $0.CreateRuleSetRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createRuleSet, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateRuleSetResponse> updateRuleSet(
    $0.UpdateRuleSetRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateRuleSet, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteRuleSetResponse> deleteRuleSet(
    $0.DeleteRuleSetRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteRuleSet, request, options: options);
  }

  $grpc.ResponseFuture<$0.PublishRuleSetResponse> publishRuleSet(
    $0.PublishRuleSetRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$publishRuleSet, request, options: options);
  }

  $grpc.ResponseFuture<$0.RollbackRuleSetResponse> rollbackRuleSet(
    $0.RollbackRuleSetRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$rollbackRuleSet, request, options: options);
  }

  $grpc.ResponseFuture<$0.EvaluateResponse> evaluate(
    $0.EvaluateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$evaluate, request, options: options);
  }

  $grpc.ResponseFuture<$0.EvaluateDryRunResponse> evaluateDryRun(
    $0.EvaluateDryRunRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$evaluateDryRun, request, options: options);
  }

  // method descriptors

  static final _$listRules =
      $grpc.ClientMethod<$0.ListRulesRequest, $0.ListRulesResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/ListRules',
          ($0.ListRulesRequest value) => value.writeToBuffer(),
          $0.ListRulesResponse.fromBuffer);
  static final _$getRule =
      $grpc.ClientMethod<$0.GetRuleRequest, $0.GetRuleResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/GetRule',
          ($0.GetRuleRequest value) => value.writeToBuffer(),
          $0.GetRuleResponse.fromBuffer);
  static final _$createRule =
      $grpc.ClientMethod<$0.CreateRuleRequest, $0.CreateRuleResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/CreateRule',
          ($0.CreateRuleRequest value) => value.writeToBuffer(),
          $0.CreateRuleResponse.fromBuffer);
  static final _$updateRule =
      $grpc.ClientMethod<$0.UpdateRuleRequest, $0.UpdateRuleResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/UpdateRule',
          ($0.UpdateRuleRequest value) => value.writeToBuffer(),
          $0.UpdateRuleResponse.fromBuffer);
  static final _$deleteRule =
      $grpc.ClientMethod<$0.DeleteRuleRequest, $0.DeleteRuleResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/DeleteRule',
          ($0.DeleteRuleRequest value) => value.writeToBuffer(),
          $0.DeleteRuleResponse.fromBuffer);
  static final _$listRuleSets =
      $grpc.ClientMethod<$0.ListRuleSetsRequest, $0.ListRuleSetsResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/ListRuleSets',
          ($0.ListRuleSetsRequest value) => value.writeToBuffer(),
          $0.ListRuleSetsResponse.fromBuffer);
  static final _$getRuleSet =
      $grpc.ClientMethod<$0.GetRuleSetRequest, $0.GetRuleSetResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/GetRuleSet',
          ($0.GetRuleSetRequest value) => value.writeToBuffer(),
          $0.GetRuleSetResponse.fromBuffer);
  static final _$createRuleSet =
      $grpc.ClientMethod<$0.CreateRuleSetRequest, $0.CreateRuleSetResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/CreateRuleSet',
          ($0.CreateRuleSetRequest value) => value.writeToBuffer(),
          $0.CreateRuleSetResponse.fromBuffer);
  static final _$updateRuleSet =
      $grpc.ClientMethod<$0.UpdateRuleSetRequest, $0.UpdateRuleSetResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/UpdateRuleSet',
          ($0.UpdateRuleSetRequest value) => value.writeToBuffer(),
          $0.UpdateRuleSetResponse.fromBuffer);
  static final _$deleteRuleSet =
      $grpc.ClientMethod<$0.DeleteRuleSetRequest, $0.DeleteRuleSetResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/DeleteRuleSet',
          ($0.DeleteRuleSetRequest value) => value.writeToBuffer(),
          $0.DeleteRuleSetResponse.fromBuffer);
  static final _$publishRuleSet =
      $grpc.ClientMethod<$0.PublishRuleSetRequest, $0.PublishRuleSetResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/PublishRuleSet',
          ($0.PublishRuleSetRequest value) => value.writeToBuffer(),
          $0.PublishRuleSetResponse.fromBuffer);
  static final _$rollbackRuleSet =
      $grpc.ClientMethod<$0.RollbackRuleSetRequest, $0.RollbackRuleSetResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/RollbackRuleSet',
          ($0.RollbackRuleSetRequest value) => value.writeToBuffer(),
          $0.RollbackRuleSetResponse.fromBuffer);
  static final _$evaluate =
      $grpc.ClientMethod<$0.EvaluateRequest, $0.EvaluateResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/Evaluate',
          ($0.EvaluateRequest value) => value.writeToBuffer(),
          $0.EvaluateResponse.fromBuffer);
  static final _$evaluateDryRun =
      $grpc.ClientMethod<$0.EvaluateDryRunRequest, $0.EvaluateDryRunResponse>(
          '/k1s0.system.ruleengine.v1.RuleEngineService/EvaluateDryRun',
          ($0.EvaluateDryRunRequest value) => value.writeToBuffer(),
          $0.EvaluateDryRunResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.ruleengine.v1.RuleEngineService')
abstract class RuleEngineServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.ruleengine.v1.RuleEngineService';

  RuleEngineServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.ListRulesRequest, $0.ListRulesResponse>(
        'ListRules',
        listRules_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListRulesRequest.fromBuffer(value),
        ($0.ListRulesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetRuleRequest, $0.GetRuleResponse>(
        'GetRule',
        getRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetRuleRequest.fromBuffer(value),
        ($0.GetRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateRuleRequest, $0.CreateRuleResponse>(
        'CreateRule',
        createRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CreateRuleRequest.fromBuffer(value),
        ($0.CreateRuleResponse value) => value.writeToBuffer()));
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
    $addMethod(
        $grpc.ServiceMethod<$0.ListRuleSetsRequest, $0.ListRuleSetsResponse>(
            'ListRuleSets',
            listRuleSets_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListRuleSetsRequest.fromBuffer(value),
            ($0.ListRuleSetsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetRuleSetRequest, $0.GetRuleSetResponse>(
        'GetRuleSet',
        getRuleSet_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetRuleSetRequest.fromBuffer(value),
        ($0.GetRuleSetResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.CreateRuleSetRequest, $0.CreateRuleSetResponse>(
            'CreateRuleSet',
            createRuleSet_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateRuleSetRequest.fromBuffer(value),
            ($0.CreateRuleSetResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpdateRuleSetRequest, $0.UpdateRuleSetResponse>(
            'UpdateRuleSet',
            updateRuleSet_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpdateRuleSetRequest.fromBuffer(value),
            ($0.UpdateRuleSetResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteRuleSetRequest, $0.DeleteRuleSetResponse>(
            'DeleteRuleSet',
            deleteRuleSet_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteRuleSetRequest.fromBuffer(value),
            ($0.DeleteRuleSetResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.PublishRuleSetRequest,
            $0.PublishRuleSetResponse>(
        'PublishRuleSet',
        publishRuleSet_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.PublishRuleSetRequest.fromBuffer(value),
        ($0.PublishRuleSetResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RollbackRuleSetRequest,
            $0.RollbackRuleSetResponse>(
        'RollbackRuleSet',
        rollbackRuleSet_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RollbackRuleSetRequest.fromBuffer(value),
        ($0.RollbackRuleSetResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.EvaluateRequest, $0.EvaluateResponse>(
        'Evaluate',
        evaluate_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.EvaluateRequest.fromBuffer(value),
        ($0.EvaluateResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.EvaluateDryRunRequest,
            $0.EvaluateDryRunResponse>(
        'EvaluateDryRun',
        evaluateDryRun_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.EvaluateDryRunRequest.fromBuffer(value),
        ($0.EvaluateDryRunResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.ListRulesResponse> listRules_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListRulesRequest> $request) async {
    return listRules($call, await $request);
  }

  $async.Future<$0.ListRulesResponse> listRules(
      $grpc.ServiceCall call, $0.ListRulesRequest request);

  $async.Future<$0.GetRuleResponse> getRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetRuleRequest> $request) async {
    return getRule($call, await $request);
  }

  $async.Future<$0.GetRuleResponse> getRule(
      $grpc.ServiceCall call, $0.GetRuleRequest request);

  $async.Future<$0.CreateRuleResponse> createRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateRuleRequest> $request) async {
    return createRule($call, await $request);
  }

  $async.Future<$0.CreateRuleResponse> createRule(
      $grpc.ServiceCall call, $0.CreateRuleRequest request);

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

  $async.Future<$0.ListRuleSetsResponse> listRuleSets_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListRuleSetsRequest> $request) async {
    return listRuleSets($call, await $request);
  }

  $async.Future<$0.ListRuleSetsResponse> listRuleSets(
      $grpc.ServiceCall call, $0.ListRuleSetsRequest request);

  $async.Future<$0.GetRuleSetResponse> getRuleSet_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetRuleSetRequest> $request) async {
    return getRuleSet($call, await $request);
  }

  $async.Future<$0.GetRuleSetResponse> getRuleSet(
      $grpc.ServiceCall call, $0.GetRuleSetRequest request);

  $async.Future<$0.CreateRuleSetResponse> createRuleSet_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateRuleSetRequest> $request) async {
    return createRuleSet($call, await $request);
  }

  $async.Future<$0.CreateRuleSetResponse> createRuleSet(
      $grpc.ServiceCall call, $0.CreateRuleSetRequest request);

  $async.Future<$0.UpdateRuleSetResponse> updateRuleSet_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateRuleSetRequest> $request) async {
    return updateRuleSet($call, await $request);
  }

  $async.Future<$0.UpdateRuleSetResponse> updateRuleSet(
      $grpc.ServiceCall call, $0.UpdateRuleSetRequest request);

  $async.Future<$0.DeleteRuleSetResponse> deleteRuleSet_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteRuleSetRequest> $request) async {
    return deleteRuleSet($call, await $request);
  }

  $async.Future<$0.DeleteRuleSetResponse> deleteRuleSet(
      $grpc.ServiceCall call, $0.DeleteRuleSetRequest request);

  $async.Future<$0.PublishRuleSetResponse> publishRuleSet_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.PublishRuleSetRequest> $request) async {
    return publishRuleSet($call, await $request);
  }

  $async.Future<$0.PublishRuleSetResponse> publishRuleSet(
      $grpc.ServiceCall call, $0.PublishRuleSetRequest request);

  $async.Future<$0.RollbackRuleSetResponse> rollbackRuleSet_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RollbackRuleSetRequest> $request) async {
    return rollbackRuleSet($call, await $request);
  }

  $async.Future<$0.RollbackRuleSetResponse> rollbackRuleSet(
      $grpc.ServiceCall call, $0.RollbackRuleSetRequest request);

  $async.Future<$0.EvaluateResponse> evaluate_Pre($grpc.ServiceCall $call,
      $async.Future<$0.EvaluateRequest> $request) async {
    return evaluate($call, await $request);
  }

  $async.Future<$0.EvaluateResponse> evaluate(
      $grpc.ServiceCall call, $0.EvaluateRequest request);

  $async.Future<$0.EvaluateDryRunResponse> evaluateDryRun_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.EvaluateDryRunRequest> $request) async {
    return evaluateDryRun($call, await $request);
  }

  $async.Future<$0.EvaluateDryRunResponse> evaluateDryRun(
      $grpc.ServiceCall call, $0.EvaluateDryRunRequest request);
}
