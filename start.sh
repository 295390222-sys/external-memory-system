#!/bin/bash

# 外部记忆系统启动脚本
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

# 检查服务是否运行
check_service() {
    if [ -f "server.pid" ]; then
        PID=$(cat server.pid)
        if ps -p $PID > /dev/null; then
            return 0
        else
            rm -f server.pid
            return 1
        fi
    else
        return 1
    fi
}

# 启动服务
start_service() {
    log_info "启动gRPC服务器..."
    
    # 激活虚拟环境
    if [ -d "venv" ]; then
        source venv/bin/activate
        log_info "虚拟环境已激活"
    fi
    
    # 启动服务器
    cd memory-server
    nohup python server.py --db ../database/memory.db --port 50051 > ../server.log 2>&1 &
    echo $! > ../server.pid
    cd ..
    
    # 等待服务启动
    sleep 3
    
    # 检查服务状态
    if check_service; then
        log_success "gRPC服务器启动成功 (PID: $(cat server.pid))"
        log_info "服务地址: localhost:50051"
        log_info "日志文件: server.log"
    else
        log_error "gRPC服务器启动失败"
        log_error "请检查日志文件: server.log"
        exit 1
    fi
}

# 启动Docker容器（可选）
start_docker() {
    log_info "启动Docker容器..."
    
    if command -v docker-compose &> /dev/null; then
        docker-compose up -d
        log_success "Docker容器启动成功"
    else
        log_warning "Docker Compose未安装，跳过Docker启动"
    fi
}

# 主函数
main() {
    log_info "启动外部记忆系统..."
    
    # 检查服务是否已运行
    if check_service; then
        log_warning "服务已在运行 (PID: $(cat server.pid))"
        log_info "使用 './stop.sh' 停止服务"
        exit 0
    fi
    
    # 确保数据库存在
    if [ ! -f "database/memory.db" ]; then
        log_info "初始化数据库..."
        mkdir -p database
        sqlite3 database/memory.db "VACUUM;"
    fi
    
    # 启动服务
    start_service
    
    # 启动Docker容器（如果需要）
    if [ "$1" = "--docker" ]; then
        start_docker
    fi
    
    log_success "外部记忆系统启动完成！"
    
    # 显示服务信息
    echo ""
    echo "服务信息:"
    echo "  - gRPC端口: 50051"
    echo "  - 数据库: database/memory.db"
    echo "  - 日志: server.log"
    echo "  - PID: $(cat server.pid)"
    echo ""
    echo "使用命令:"
    echo "  - './stop.sh' - 停止服务"
    echo "  - './status.sh' - 查看状态"
    echo "  - './logs.sh' - 查看日志"
}

# 执行主函数
main "$@"