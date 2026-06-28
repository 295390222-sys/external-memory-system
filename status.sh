#!/bin/bash

# 外部记忆系统状态检查脚本
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

# 检查服务状态
check_service_status() {
    echo "=== 服务状态检查 ==="
    
    if [ -f "server.pid" ]; then
        PID=$(cat server.pid)
        if ps -p $PID > /dev/null; then
            log_success "gRPC服务器正在运行 (PID: $PID)"
            
            # 检查端口
            if netstat -tuln | grep -q ":50051 "; then
                log_success "端口50051已开放"
            else
                log_error "端口50051未开放"
            fi
            
            # 检查进程资源使用
            echo ""
            echo "进程资源使用:"
            echo "  CPU: $(ps -p $PID -o %cpu --no-headers)%"
            echo "  内存: $(ps -p $PID -o rss --no-headers)KB"
            echo "  运行时间: $(ps -p $PID -o etimes --no-headers)秒"
            
        else
            log_error "gRPC服务器进程不存在 (PID: $PID)"
            log_warning "PID文件可能过时"
        fi
    else
        log_error "未找到服务器PID文件"
        log_warning "服务可能未启动"
    fi
}

# 检查数据库状态
check_database_status() {
    echo ""
    echo "=== 数据库状态检查 ==="
    
    if [ -f "database/memory.db" ]; then
        log_success "数据库文件存在"
        
        # 检查数据库大小
        DB_SIZE=$(stat -f%z database/memory.db 2>/dev/null || stat -c%s database/memory.db 2>/dev/null)
        echo "  数据库大小: $DB_SIZE 字节"
        
        # 检查数据库表
        echo ""
        echo "数据库表信息:"
        sqlite3 database/memory.db ".tables" 2>/dev/null || echo "  无法连接到数据库"
        
    else
        log_error "数据库文件不存在"
    fi
}

# 检查Docker状态
check_docker_status() {
    echo ""
    echo "=== Docker状态检查 ==="
    
    if command -v docker-compose &> /dev/null; then
        if docker-compose ps 2>/dev/null; then
            log_success "Docker容器运行正常"
            docker-compose ps
        else
            log_warning "Docker容器未运行"
        fi
    else
        log_warning "Docker Compose未安装"
    fi
}

# 检查端口状态
check_ports() {
    echo ""
    echo "=== 端口状态检查 ==="
    
    ports=("50051" "50052" "6333" "6334")
    
    for port in "${ports[@]}"; do
        if netstat -tuln | grep -q ":$port "; then
            log_success "端口 $port 已开放"
        else
            log_warning "端口 $port 未开放"
        fi
    done
}

# 检查磁盘空间
check_disk_space() {
    echo ""
    echo "=== 磁盘空间检查 ==="
    
    # 检查当前目录磁盘使用情况
    if command -v df &> /dev/null; then
        df -h .
    fi
    
    # 检查数据库目录大小
    if [ -d "database" ]; then
        DB_DIR_SIZE=$(du -sh database 2>/dev/null | cut -f1)
        echo "数据库目录大小: $DB_DIR_SIZE"
    fi
}

# 检查日志文件
check_logs() {
    echo ""
    echo "=== 日志文件检查 ==="
    
    if [ -f "server.log" ]; then
        LOG_SIZE=$(stat -f%z server.log 2>/dev/null || stat -c%s server.log 2>/dev/null)
        echo "日志文件大小: $LOG_SIZE 字节"
        
        # 显示最近的日志
        echo ""
        echo "最近的日志:"
        tail -n 10 server.log
    else
        log_warning "日志文件不存在"
    fi
}

# 主函数
main() {
    echo "外部记忆系统状态检查"
    echo "检查时间: $(date)"
    echo ""
    
    # 检查服务状态
    check_service_status
    
    # 检查数据库状态
    check_database_status
    
    # 检查Docker状态
    check_docker_status
    
    # 检查端口状态
    check_ports
    
    # 检查磁盘空间
    check_disk_space
    
    # 检查日志文件
    check_logs
    
    echo ""
    echo "=== 检查完成 ==="
}

# 执行主函数
main "$@"