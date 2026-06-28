#!/bin/bash

# 外部记忆系统停止脚本
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

# 停止服务
stop_service() {
    if [ -f "server.pid" ]; then
        PID=$(cat server.pid)
        if ps -p $PID > /dev/null; then
            log_info "停止gRPC服务器 (PID: $PID)..."
            kill $PID
            
            # 等待进程结束
            for i in {1..10}; do
                if ! ps -p $PID > /dev/null; then
                    break
                fi
                sleep 1
            done
            
            # 如果进程还在运行，强制终止
            if ps -p $PID > /dev/null; then
                log_warning "强制终止进程..."
                kill -9 $PID
            fi
            
            rm -f server.pid
            log_success "gRPC服务器已停止"
        else
            log_warning "进程 $PID 不存在"
            rm -f server.pid
        fi
    else
        log_warning "未找到服务器PID文件"
    fi
}

# 停止Docker容器
stop_docker() {
    log_info "停止Docker容器..."
    
    if command -v docker-compose &> /dev/null; then
        docker-compose down
        log_success "Docker容器已停止"
    else
        log_warning "Docker Compose未安装，跳过Docker停止"
    fi
}

# 主函数
main() {
    log_info "停止外部记忆系统..."
    
    # 停止服务
    stop_service
    
    # 停止Docker容器
    stop_docker
    
    log_success "外部记忆系统已停止！"
    
    # 清理临时文件
    if [ "$1" = "--clean" ]; then
        log_info "清理临时文件..."
        rm -f *.log *.pid
        log_success "临时文件已清理"
    fi
}

# 执行主函数
main "$@"