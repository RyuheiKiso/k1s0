import { Suspense, lazy, Component, type ReactNode } from "react";
import { Layout, Menu, Spin, theme } from "antd";
import {
  AppstoreAddOutlined,
  BranchesOutlined,
  DatabaseOutlined,
  FileExcelOutlined,
  SafetyCertificateOutlined,
} from "@ant-design/icons";
import { Navigate, Route, Routes, useLocation, useNavigate } from "react-router-dom";
const DashboardPage = lazy(() => import("./pages/DashboardPage").then((module) => ({ default: module.DashboardPage })));
const TableWorkbenchPage = lazy(() =>
  import("./pages/TableWorkbenchPage").then((module) => ({ default: module.TableWorkbenchPage }))
);
const RuleConsolePage = lazy(() => import("./pages/RuleConsolePage").then((module) => ({ default: module.RuleConsolePage })));
const ImportStudioPage = lazy(() =>
  import("./pages/ImportStudioPage").then((module) => ({ default: module.ImportStudioPage }))
);
const RelationshipMapPage = lazy(() =>
  import("./pages/RelationshipMapPage").then((module) => ({ default: module.RelationshipMapPage }))
);

// ErrorBoundaryのProps定義
interface ErrorBoundaryProps {
  children: ReactNode;
}

// ErrorBoundaryの内部状態
interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

// エラーバウンダリコンポーネント: 子コンポーネントのレンダリングエラーをキャッチして表示
class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  // レンダリングエラー発生時にエラー状態を更新
  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  override render() {
    // エラー発生時はフォールバックUIを表示
    if (this.state.hasError) {
      return (
        <div role="alert" style={{ padding: 24 }}>
          <h2>エラーが発生しました</h2>
          <p>{this.state.error?.message}</p>
        </div>
      );
    }
    return this.props.children;
  }
}

const items = [
  { key: "/dashboard", icon: <AppstoreAddOutlined />, label: "Dashboard" },
  { key: "/tables", icon: <DatabaseOutlined />, label: "Tables" },
  { key: "/rules", icon: <SafetyCertificateOutlined />, label: "Rules" },
  { key: "/relationships", icon: <BranchesOutlined />, label: "Relationships" },
  { key: "/imports", icon: <FileExcelOutlined />, label: "Imports" },
];

export function App() {
  const navigate = useNavigate();
  const location = useLocation();
  const { token } = theme.useToken();

  return (
    <ErrorBoundary>
    <Layout style={{ minHeight: "100vh", background: token.colorBgLayout }}>
      <Layout.Sider breakpoint="lg" collapsedWidth={0} width={264} theme="light">
        <div className="brand-block">
          <p className="brand-kicker">system tier console</p>
          <h1>Master Maintenance</h1>
          <p className="brand-copy">metadata, rules, imports, and record control in one surface.</p>
        </div>
        <Menu
          mode="inline"
          selectedKeys={[location.pathname]}
          items={items}
          onClick={({ key }) => navigate(key)}
        />
      </Layout.Sider>
      <Layout>
        <Layout.Header className="app-header">
          <div>
            <span className="mono-label">control plane</span>
            <strong>metadata-driven operations</strong>
          </div>
        </Layout.Header>
        <Layout.Content style={{ padding: 24 }}>
          <Suspense fallback={<div className="route-loader"><Spin size="large" /></div>}>
            <Routes>
              <Route path="/" element={<Navigate to="/dashboard" replace />} />
              <Route path="/dashboard" element={<DashboardPage />} />
              <Route path="/tables" element={<TableWorkbenchPage />} />
              <Route path="/rules" element={<RuleConsolePage />} />
              <Route path="/relationships" element={<RelationshipMapPage />} />
              <Route path="/imports" element={<ImportStudioPage />} />
            </Routes>
          </Suspense>
        </Layout.Content>
      </Layout>
    </Layout>
    </ErrorBoundary>
  );
}
