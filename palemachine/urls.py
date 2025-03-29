"""
URL configuration for palemachine project.

The `urlpatterns` list routes URLs to views. For more information please see:
    https://docs.djangoproject.com/en/5.1/topics/http/urls/
Examples:
Function views
    1. Add an import:  from my_app import views
    2. Add a URL to urlpatterns:  path('', views.home, name='home')
Class-based views
    1. Add an import:  from other_app.views import Home
Including another URLconf
    1. Import the include() function: from django.urls import include, path
    2. Add a URL to urlpatterns:  path('blog/', include('blog.urls'))
"""
from django.contrib import admin
from django.urls import path
from .views import *

urlpatterns = [
    path('admin/', admin.site.urls),
    path('instantdl/', instantdl, name='instantdl'),
    path('manualdl/', manualdl, name='manualdl'),
    path('imgquestion/', imgquestion, name='imgquestion'),
    path('spotify_result_json/', spotify_result_json, name='spotify_result_json'),
    path('', download_form, name='download_form'),
    
]
