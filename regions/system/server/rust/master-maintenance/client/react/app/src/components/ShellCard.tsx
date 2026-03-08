import { Card } from "antd";
import type { PropsWithChildren, ReactNode } from "react";

export function ShellCard({
  title,
  extra,
  children,
}: PropsWithChildren<{ title: string; extra?: ReactNode }>) {
  return (
    <Card
      title={title}
      extra={extra}
      styles={{ body: { padding: 20 } }}
      style={{ borderRadius: 24, boxShadow: "0 14px 42px rgba(15, 23, 42, 0.08)" }}
    >
      {children}
    </Card>
  );
}
