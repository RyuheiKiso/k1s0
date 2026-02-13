use std::path::PathBuf;

use crate::domain::model::{Database, ProjectConfig, Template};

struct K8sParams {
    name: String,
    container_port: u16,
    health_path: String,
    cpu_request: String,
    cpu_limit: String,
    mem_request: String,
    mem_limit: String,
    has_postgres: bool,
}

impl K8sParams {
    fn from_config(config: &ProjectConfig) -> Self {
        let (container_port, health_path, cpu_request, cpu_limit, mem_request, mem_limit) =
            match &config.template {
                Template::React | Template::Flutter => {
                    (80, "/", "50m", "200m", "64Mi", "128Mi")
                }
                Template::RustAxum => (3000, "/health", "100m", "500m", "128Mi", "256Mi"),
                Template::GoGin => (8080, "/health", "100m", "500m", "128Mi", "256Mi"),
            };

        Self {
            name: config.name.clone(),
            container_port,
            health_path: health_path.to_string(),
            cpu_request: cpu_request.to_string(),
            cpu_limit: cpu_limit.to_string(),
            mem_request: mem_request.to_string(),
            mem_limit: mem_limit.to_string(),
            has_postgres: config.database == Database::PostgreSql,
        }
    }
}

pub fn generate(config: &ProjectConfig) -> Vec<(PathBuf, String)> {
    let params = K8sParams::from_config(config);
    let mut files = vec![
        (PathBuf::from("k8s/namespace.yml"), namespace(&params)),
        (PathBuf::from("k8s/deployment.yml"), deployment(&params)),
        (PathBuf::from("k8s/service.yml"), service(&params)),
        (PathBuf::from("k8s/ingress.yml"), ingress(&params)),
        (PathBuf::from("k8s/configmap.yml"), configmap(&params)),
    ];

    if params.has_postgres {
        files.push((PathBuf::from("k8s/postgres-secret.yml"), postgres_secret(&params)));
        files.push((PathBuf::from("k8s/postgres-pvc.yml"), postgres_pvc(&params)));
        files.push((
            PathBuf::from("k8s/postgres-statefulset.yml"),
            postgres_statefulset(&params),
        ));
        files.push((
            PathBuf::from("k8s/postgres-service.yml"),
            postgres_service(&params),
        ));
    }

    files
}

fn namespace(params: &K8sParams) -> String {
    format!(
        r#"apiVersion: v1
kind: Namespace
metadata:
  name: {name}
"#,
        name = params.name
    )
}

fn deployment(params: &K8sParams) -> String {
    let env_section = if params.has_postgres {
        format!(
            r#"          envFrom:
            - configMapRef:
                name: {name}-config
            - secretRef:
                name: {name}-postgres-secret"#,
            name = params.name
        )
    } else {
        format!(
            r#"          envFrom:
            - configMapRef:
                name: {name}-config"#,
            name = params.name
        )
    };

    format!(
        r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {name}
  namespace: {name}
  labels:
    app: {name}
spec:
  replicas: 2
  selector:
    matchLabels:
      app: {name}
  template:
    metadata:
      labels:
        app: {name}
    spec:
      containers:
        - name: {name}
          image: {name}:latest
          ports:
            - containerPort: {port}
{env_section}
          resources:
            requests:
              cpu: {cpu_req}
              memory: {mem_req}
            limits:
              cpu: {cpu_lim}
              memory: {mem_lim}
          livenessProbe:
            httpGet:
              path: {health}
              port: {port}
            initialDelaySeconds: 10
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: {health}
              port: {port}
            initialDelaySeconds: 5
            periodSeconds: 10
"#,
        name = params.name,
        port = params.container_port,
        env_section = env_section,
        cpu_req = params.cpu_request,
        cpu_lim = params.cpu_limit,
        mem_req = params.mem_request,
        mem_lim = params.mem_limit,
        health = params.health_path,
    )
}

fn service(params: &K8sParams) -> String {
    format!(
        r#"apiVersion: v1
kind: Service
metadata:
  name: {name}
  namespace: {name}
spec:
  type: ClusterIP
  selector:
    app: {name}
  ports:
    - port: {port}
      targetPort: {port}
      protocol: TCP
"#,
        name = params.name,
        port = params.container_port,
    )
}

fn ingress(params: &K8sParams) -> String {
    format!(
        r#"apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {name}
  namespace: {name}
  annotations:
    kubernetes.io/ingress.class: nginx
spec:
  rules:
    - host: {name}.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: {name}
                port:
                  number: {port}
"#,
        name = params.name,
        port = params.container_port,
    )
}

fn configmap(params: &K8sParams) -> String {
    let data = if params.has_postgres {
        format!(
            r#"  APP_NAME: {name}
  APP_ENV: production
  DATABASE_HOST: {name}-postgres
  DATABASE_PORT: "5432"
  DATABASE_NAME: app_db"#,
            name = params.name
        )
    } else {
        format!(
            r#"  APP_NAME: {name}
  APP_ENV: production"#,
            name = params.name
        )
    };

    format!(
        r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: {name}-config
  namespace: {name}
data:
{data}
"#,
        name = params.name,
        data = data,
    )
}

fn postgres_secret(params: &K8sParams) -> String {
    format!(
        r#"apiVersion: v1
kind: Secret
metadata:
  name: {name}-postgres-secret
  namespace: {name}
type: Opaque
stringData:
  POSTGRES_USER: app
  POSTGRES_PASSWORD: password
  POSTGRES_DB: app_db
  DATABASE_URL: postgres://app:password@{name}-postgres:5432/app_db
"#,
        name = params.name
    )
}

fn postgres_pvc(params: &K8sParams) -> String {
    format!(
        r#"apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: {name}-postgres-data
  namespace: {name}
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
"#,
        name = params.name
    )
}

fn postgres_statefulset(params: &K8sParams) -> String {
    format!(
        r#"apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: {name}-postgres
  namespace: {name}
spec:
  serviceName: {name}-postgres
  replicas: 1
  selector:
    matchLabels:
      app: {name}-postgres
  template:
    metadata:
      labels:
        app: {name}-postgres
    spec:
      containers:
        - name: postgres
          image: postgres:16
          ports:
            - containerPort: 5432
          envFrom:
            - secretRef:
                name: {name}-postgres-secret
          volumeMounts:
            - name: postgres-data
              mountPath: /var/lib/postgresql/data
      volumes:
        - name: postgres-data
          persistentVolumeClaim:
            claimName: {name}-postgres-data
"#,
        name = params.name
    )
}

fn postgres_service(params: &K8sParams) -> String {
    format!(
        r#"apiVersion: v1
kind: Service
metadata:
  name: {name}-postgres
  namespace: {name}
spec:
  type: ClusterIP
  selector:
    app: {name}-postgres
  ports:
    - port: 5432
      targetPort: 5432
      protocol: TCP
"#,
        name = params.name
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{ProjectConfig, ProjectType};

    fn backend_config(template: Template, database: Database) -> ProjectConfig {
        ProjectConfig {
            name: "test-svc".to_string(),
            project_type: ProjectType::Backend,
            template,
            database,
            path: PathBuf::from("/tmp/test-svc"),
        }
    }

    fn frontend_config() -> ProjectConfig {
        ProjectConfig {
            name: "test-app".to_string(),
            project_type: ProjectType::Frontend,
            template: Template::React,
            database: Database::None,
            path: PathBuf::from("/tmp/test-app"),
        }
    }

    #[test]
    fn test_generate_base_files_count() {
        let config = backend_config(Template::RustAxum, Database::None);
        let files = generate(&config);
        assert_eq!(files.len(), 5);
    }

    #[test]
    fn test_generate_with_postgres_files_count() {
        let config = backend_config(Template::RustAxum, Database::PostgreSql);
        let files = generate(&config);
        assert_eq!(files.len(), 9);
    }

    #[test]
    fn test_generate_base_file_paths() {
        let config = backend_config(Template::GoGin, Database::None);
        let paths: Vec<PathBuf> = generate(&config).into_iter().map(|(p, _)| p).collect();
        assert!(paths.contains(&PathBuf::from("k8s/namespace.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/deployment.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/service.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/ingress.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/configmap.yml")));
    }

    #[test]
    fn test_generate_postgres_file_paths() {
        let config = backend_config(Template::RustAxum, Database::PostgreSql);
        let paths: Vec<PathBuf> = generate(&config).into_iter().map(|(p, _)| p).collect();
        assert!(paths.contains(&PathBuf::from("k8s/postgres-secret.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/postgres-pvc.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/postgres-statefulset.yml")));
        assert!(paths.contains(&PathBuf::from("k8s/postgres-service.yml")));
    }

    #[test]
    fn test_namespace_content() {
        let config = backend_config(Template::RustAxum, Database::None);
        let params = K8sParams::from_config(&config);
        let content = namespace(&params);
        assert!(content.contains("kind: Namespace"));
        assert!(content.contains("name: test-svc"));
    }

    #[test]
    fn test_deployment_rust_axum() {
        let config = backend_config(Template::RustAxum, Database::None);
        let params = K8sParams::from_config(&config);
        let content = deployment(&params);
        assert!(content.contains("containerPort: 3000"));
        assert!(content.contains("path: /health"));
        assert!(content.contains("cpu: 100m"));
        assert!(content.contains("memory: 128Mi"));
    }

    #[test]
    fn test_deployment_go_gin() {
        let config = backend_config(Template::GoGin, Database::None);
        let params = K8sParams::from_config(&config);
        let content = deployment(&params);
        assert!(content.contains("containerPort: 8080"));
        assert!(content.contains("path: /health"));
    }

    #[test]
    fn test_deployment_react() {
        let config = frontend_config();
        let params = K8sParams::from_config(&config);
        let content = deployment(&params);
        assert!(content.contains("containerPort: 80"));
        assert!(content.contains("path: /"));
        assert!(content.contains("cpu: 50m"));
        assert!(content.contains("memory: 64Mi"));
    }

    #[test]
    fn test_deployment_with_postgres_env() {
        let config = backend_config(Template::RustAxum, Database::PostgreSql);
        let params = K8sParams::from_config(&config);
        let content = deployment(&params);
        assert!(content.contains("secretRef"));
        assert!(content.contains("test-svc-postgres-secret"));
    }

    #[test]
    fn test_service_content() {
        let config = backend_config(Template::RustAxum, Database::None);
        let params = K8sParams::from_config(&config);
        let content = service(&params);
        assert!(content.contains("type: ClusterIP"));
        assert!(content.contains("port: 3000"));
    }

    #[test]
    fn test_ingress_content() {
        let config = backend_config(Template::RustAxum, Database::None);
        let params = K8sParams::from_config(&config);
        let content = ingress(&params);
        assert!(content.contains("kind: Ingress"));
        assert!(content.contains("host: test-svc.local"));
        assert!(content.contains("ingress.class: nginx"));
    }

    #[test]
    fn test_configmap_with_postgres() {
        let config = backend_config(Template::RustAxum, Database::PostgreSql);
        let params = K8sParams::from_config(&config);
        let content = configmap(&params);
        assert!(content.contains("DATABASE_HOST"));
        assert!(content.contains("DATABASE_PORT"));
    }

    #[test]
    fn test_configmap_without_postgres() {
        let config = backend_config(Template::GoGin, Database::None);
        let params = K8sParams::from_config(&config);
        let content = configmap(&params);
        assert!(content.contains("APP_NAME: test-svc"));
        assert!(!content.contains("DATABASE_HOST"));
    }

    #[test]
    fn test_postgres_secret_content() {
        let config = backend_config(Template::RustAxum, Database::PostgreSql);
        let params = K8sParams::from_config(&config);
        let content = postgres_secret(&params);
        assert!(content.contains("kind: Secret"));
        assert!(content.contains("POSTGRES_USER: app"));
        assert!(content.contains("POSTGRES_PASSWORD: password"));
        assert!(content.contains("DATABASE_URL"));
    }

    #[test]
    fn test_postgres_pvc_content() {
        let config = backend_config(Template::RustAxum, Database::PostgreSql);
        let params = K8sParams::from_config(&config);
        let content = postgres_pvc(&params);
        assert!(content.contains("kind: PersistentVolumeClaim"));
        assert!(content.contains("storage: 1Gi"));
    }

    #[test]
    fn test_postgres_statefulset_content() {
        let config = backend_config(Template::RustAxum, Database::PostgreSql);
        let params = K8sParams::from_config(&config);
        let content = postgres_statefulset(&params);
        assert!(content.contains("kind: StatefulSet"));
        assert!(content.contains("postgres:16"));
        assert!(content.contains("containerPort: 5432"));
    }
}
