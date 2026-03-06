import { Col, Progress, Row, Statistic, Tag, Typography } from "antd";
import { ShellCard } from "../components/ShellCard";
import { useTables } from "../hooks";

export function DashboardPage() {
  const { data, isLoading } = useTables();
  const tables = data?.tables ?? [];
  const activeCount = tables.filter((table) => table.is_active).length;
  const writableCount = tables.filter((table) => table.allow_update).length;
  const destructiveCount = tables.filter((table) => table.allow_delete).length;

  return (
    <div className="page-stack">
      <section className="hero-panel">
        <div>
          <Typography.Text className="mono-label">state snapshot</Typography.Text>
          <Typography.Title level={2} style={{ marginTop: 10 }}>
            System metadata stays controllable only when structure, rule, and import drift are visible.
          </Typography.Title>
        </div>
        <div className="hero-metrics">
          <div>
            <span>active tables</span>
            <strong>{isLoading ? "..." : activeCount}</strong>
          </div>
          <div>
            <span>writable</span>
            <strong>{isLoading ? "..." : writableCount}</strong>
          </div>
          <div>
            <span>delete-enabled</span>
            <strong>{isLoading ? "..." : destructiveCount}</strong>
          </div>
        </div>
      </section>
      <Row gutter={[16, 16]}>
        <Col xs={24} md={8}>
          <ShellCard title="Coverage">
            <Statistic value={tables.length} suffix="tables" />
            <Progress percent={tables.length === 0 ? 0 : Math.round((activeCount / tables.length) * 100)} />
          </ShellCard>
        </Col>
        <Col xs={24} md={8}>
          <ShellCard title="Mutation Surface">
            <Statistic value={writableCount} suffix="write enabled" />
            <Typography.Paragraph type="secondary">
              Tables with update permission and operational drift potential.
            </Typography.Paragraph>
          </ShellCard>
        </Col>
        <Col xs={24} md={8}>
          <ShellCard title="High Risk">
            <Statistic value={destructiveCount} suffix="delete enabled" />
            <Tag color={destructiveCount > 0 ? "red" : "green"}>
              {destructiveCount > 0 ? "review advised" : "contained"}
            </Tag>
          </ShellCard>
        </Col>
      </Row>
    </div>
  );
}
