#!/bin/bash

# 外部记忆系统部署脚本
# 作者：煤球
# 日期：2026-06-28

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    # 检查Python
    if ! command -v python3 &> /dev/null; then
        log_error "Python 3 未安装"
        exit 1
    fi
    
    # 检查pip
    if ! command -v pip3 &> /dev/null; then
        log_error "pip3 未安装"
        exit 1
    fi
    
    # 检查gRPC
    if ! python3 -c "import grpc" &> /dev/null; then
        log_warning "gRPC 未安装，正在安装..."
        pip3 install grpcio grpcio-tools
    fi
    
    log_success "依赖检查完成"
}

# 创建虚拟环境
create_venv() {
    log_info "创建虚拟环境..."
    
    if [ ! -d "venv" ]; then
        python3 -m venv venv
        log_success "虚拟环境创建完成"
    else
        log_warning "虚拟环境已存在，跳过创建"
    fi
    
    # 激活虚拟环境
    source venv/bin/activate
    
    # 安装依赖
    log_info "安装Python依赖..."
    pip install -r memory-plugin/requirements.txt
    pip install -r memory-core-rust/requirements.txt
    
    log_success "依赖安装完成"
}

# 生成protobuf文件
generate_proto() {
    log_info "生成protobuf文件..."
    
    cd proto
    python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. memory.proto
    cd ..
    
    # 复制生成的文件到各个目录
    cp proto/memory_pb2.py memory-server/
    cp proto/memory_pb2_grpc.py memory-server/
    cp proto/memory_pb2.py memory-plugin/
    cp proto/memory_pb2_grpc.py memory-plugin/
    
    log_success "protobuf文件生成完成"
}

# 初始化数据库
init_database() {
    log_info "初始化数据库..."
    
    # 创建数据库目录
    mkdir -p database
    
    # 初始化SQLite数据库
    if [ ! -f "database/memory.db" ]; then
        sqlite3 database/memory.db "VACUUM;"
        log_success "数据库初始化完成"
    else
        log_warning "数据库已存在，跳过初始化"
    fi
}

# 配置文件
setup_config() {
    log_info "配置文件设置..."
    
    # 创建默认配置文件
    cat > memory-plugin/config.yaml << EOF
# 外部记忆系统配置
database:
  path: "../database/memory.db"
  type: "sqlite"

grpc:
  host: "localhost"
  port: 50051
  max_workers: 10

memory:
  default_namespace: "shared"
  max_content_length: 10000
  retention_days: 365

search:
  max_results: 100
  similarity_threshold: 0.7

dream:
  enabled: true
  frequency: "daily"
  max_dream_length: 5000
EOF

    log_success "配置文件设置完成"
}

# 启动服务
start_services() {
    log_info "启动服务..."
    
    # 启动gRPC服务器
    cd memory-server
    nohup python server.py --db ../database/memory.db --port 50051 > ../server.log 2>&1 &
    echo $! > ../server.pid
    cd ..
    
    # 等待服务启动
    sleep 3
    
    # 检查服务状态
    if ps -p $(cat server.pid) > /dev/null; then
        log_success "gRPC服务器启动成功"
    else
        log_error "gRPC服务器启动失败"
        exit 1
    fi
    
    log_success "所有服务启动完成"
}

# 停止服务
stop_services() {
    log_info "停止服务..."
    
    if [ -f "server.pid" ]; then
        PID=$(cat server.pid)
        if ps -p $PID > /dev/null; then
            kill $PID
            log_success "gRPC服务器已停止"
        fi
        rm server.pid
    else
        log_warning "未找到服务器PID文件"
    fi
}

# 检查服务状态
check_status() {
    log_info "检查服务状态..."
    
    if [ -f "server.pid" ]; then
        PID=$(cat server.pid)
        if ps -p $PID > /dev/null; then
            log_success "gRPC服务器正在运行 (PID: $PID)"
        else
            log_error "gRPC服务器未运行"
        fi
    else
        log_error "未找到服务器PID文件"
    fi
}

# 备份数据库
backup_database() {
    log_info "备份数据库..."
    
    BACKUP_DIR="backups"
    mkdir -p $BACKUP_DIR
    
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    cp database/memory.db $BACKUP_DIR/memory_backup_$TIMESTAMP.db
    
    log_success "数据库备份完成: $BACKUP_DIR/memory_backup_$TIMESTAMP.db"
}

# 清理旧备份
cleanup_backups() {
    log_info "清理旧备份..."
    
    # 保留最近7天的备份
    find backups -name "memory_backup_*.db" -mtime +7 -delete
    
    log_success "旧备份清理完成"
}

# 显示帮助信息
show_help() {
    echo "外部记忆系统部署脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  install     安装和初始化系统"
    echo "  start       启动服务"
    echo "  stop        停止服务"
    echo "  restart     重启服务"
    echo "  status      检查服务状态"
    echo "  backup      备份数据库"
    echo "  clean       清理旧备份"
    echo "  uninstall   卸载系统"
    echo "  help        显示帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 install     # 安装并初始化系统"
    echo "  $0 start       # 启动服务"
    echo "  $0 stop        # 停止服务"
    echo "  $0 restart     # 重启服务"
}

# 主函数
main() {
    case "${1:-}" in
        install)
            check_dependencies
            create_venv
            generate_proto
            init_database
            setup_config
            start_services
            log_success "系统安装完成"
            ;;
        start)
            start_services
            ;;
        stop)
            stop_services
            ;;
        restart)
            stop_services
            sleep 2
            start_services
            ;;
        status)
            check_status
            ;;
        backup)
            backup_database
            cleanup_backups
            ;;
        clean)
            cleanup_backups
            ;;
        uninstall)
            stop_services
            log_info "删除虚拟环境..."
            rm -rf venv
            log_info "删除数据库..."
            rm -rf database
            log_info "删除日志文件..."
            rm -f server.log server.pid
            log_success "系统卸载完成"
            ;;
        help|--help|-h)
            show_help
            ;;
        "")
            log_error "请指定操作"
            show_help
            exit 1
            ;;
        *)
            log_error "未知选项: $1"
            show_help
            exit 1
            ;;
    esac
}

# 执行主函数
main "$@"