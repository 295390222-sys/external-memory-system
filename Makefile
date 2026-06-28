# 外部记忆系统 Makefile
# 作者：煤球
# 日期：2026-06-28

.PHONY: help install start stop restart status test clean build deploy docker-build docker-run docker-stop

# 默认目标
help:
	@echo "外部记忆系统管理"
	@echo ""
	@echo "可用目标:"
	@echo "  install     - 安装和初始化系统"
	@echo "  start       - 启动服务"
	@echo "  stop        - 停止服务"
	@echo "  restart     - 重启服务"
	@echo "  status      - 检查服务状态"
	@echo "  test        - 运行测试"
	@echo "  clean       - 清理临时文件"
	@echo "  build       - 构建Docker镜像"
	@echo "  deploy      - 部署到生产环境"
	@echo "  docker-build - 构建Docker镜像"
	@echo "  docker-run   - 运行Docker容器"
	@echo "  docker-stop  - 停止Docker容器"

# 安装系统
install:
	@echo "正在安装外部记忆系统..."
	./deploy.sh install

# 启动服务
start:
	@echo "正在启动服务..."
	./deploy.sh start

# 停止服务
stop:
	@echo "正在停止服务..."
	./deploy.sh stop

# 重启服务
restart:
	@echo "正在重启服务..."
	./deploy.sh restart

# 检查服务状态
status:
	@echo "检查服务状态..."
	./deploy.sh status

# 运行测试
test:
	@echo "运行测试..."
	python examples/integration_test.py

# 清理临时文件
clean:
	@echo "清理临时文件..."
	find . -name "*.pyc" -delete
	find . -name "__pycache__" -type d -exec rm -rf {} +
	find . -name "*.log" -delete
	rm -rf build/
	rm -rf dist/

# 构建Docker镜像
build: docker-build

# 部署到生产环境
deploy:
	@echo "部署到生产环境..."
	docker-compose up -d

# 构建Docker镜像
docker-build:
	@echo "构建Docker镜像..."
	docker-compose build

# 运行Docker容器
docker-run:
	@echo "运行Docker容器..."
	docker-compose up -d

# 停止Docker容器
docker-stop:
	@echo "停止Docker容器..."
	docker-compose down

# 开发模式
dev:
	@echo "启动开发模式..."
	docker-compose --profile rust up -d

# Web UI模式
web:
	@echo "启动Web UI模式..."
	docker-compose --profile web up -d

# 监控日志
logs:
	@echo "显示服务日志..."
	docker-compose logs -f

# 数据库备份
backup:
	@echo "备份数据库..."
	./deploy.sh backup

# 清理旧备份
clean-backups:
	@echo "清理旧备份..."
	./deploy.sh clean

# 卸载系统
uninstall:
	@echo "卸载系统..."
	./deploy.sh uninstall

# 代码格式化
format:
	@echo "格式化代码..."
	black memory-server/ memory-plugin/
	isort memory-server/ memory-plugin/

# 代码检查
lint:
	@echo "检查代码..."
	flake8 memory-server/ memory-plugin/
	mypy memory-server/ memory-plugin/

# 安全检查
security:
	@echo "安全检查..."
	bandit -r memory-server/ memory-plugin/

# 生成文档
docs:
	@echo "生成文档..."
	sphinx-apidoc -o docs/source memory-server memory_plugin
	cd docs && make html

# 发布版本
release:
	@echo "发布版本..."
	git tag -a v1.0.0 -m "Release version 1.0.0"
	git push origin v1.0.0

# 更新依赖
update-deps:
	@echo "更新依赖..."
	pip-compile requirements.in
	pip-compile memory-plugin/requirements.in